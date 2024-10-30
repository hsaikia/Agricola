use std::env;
use std::time::{Duration, Instant};
use std::{error::Error, io};

use agricola_game::agricola::algorithms::SimulationRecord;
use agricola_game::agricola::display::{print_farm, print_resources};
use agricola_game::agricola::quantity::{Quantity, Wood};
use agricola_game::agricola::state::{State, MAX_NUM_PLAYERS};
use agricola_game::agricola::{
    actions::{Action, WeightedAction},
    algorithms::{PlayerType, AI},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::style::{Color, Style};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame, Terminal,
};

const NUM_GAMES_TO_SIMULATE: usize = 1000;
const DEPTH: Option<usize> = None;

#[derive(Clone, Copy, Debug)]
enum PlayerSelection {
    Empty,
    Human,
    MctsAI,
    TdAI,
}

const PLAYER_TYPES: [PlayerSelection; 4] = [
    PlayerSelection::Empty,
    PlayerSelection::Human,
    PlayerSelection::MctsAI,
    PlayerSelection::TdAI,
];

struct App {
    selection_x: usize,
    selection_y: usize,
    player_selections: [PlayerSelection; 4],
    state: Option<State>,
    menu_active: bool,
    num_selections_y: usize,
    move_selected: bool,
    ai: [Option<AI>; MAX_NUM_PLAYERS],
    records: Vec<SimulationRecord>,
    current_actions: Vec<WeightedAction>,
}

impl App {
    fn new() -> App {
        App {
            selection_x: 0,
            selection_y: 0,
            player_selections: [PlayerSelection::Empty; 4],
            state: None,
            menu_active: false,
            num_selections_y: 1,
            move_selected: false,
            ai: [const { None }; MAX_NUM_PLAYERS],
            records: Vec::new(),
            current_actions: Vec::new(),
        }
    }

    fn next_x(&mut self) {
        if self.menu_active {
            self.selection_x = (self.selection_x + 1) % 4;
        }
    }

    fn previous_x(&mut self) {
        if self.menu_active {
            if self.selection_x == 0 {
                self.selection_x = 3;
            } else {
                self.selection_x -= 1;
            }
        }
    }

    fn next_y(&mut self) {
        self.selection_y = (self.selection_y + 1) % self.num_selections_y;
        if self.menu_active {
            self.player_selections[self.selection_x] = PLAYER_TYPES[self.selection_y];
        }
    }

    fn previous_y(&mut self) {
        if self.selection_y == 0 {
            self.selection_y = self.num_selections_y - 1;
        } else {
            self.selection_y -= 1;
        }

        if self.menu_active {
            self.player_selections[self.selection_x] = PLAYER_TYPES[self.selection_y];
        }
    }

    fn show_menu(&self) -> String {
        let mut ret: String = "Choose the Player Types : ".to_string();
        for (i, player_selection) in self.player_selections.iter().enumerate() {
            if i == self.selection_x {
                ret.push_str(&format!("[{player_selection:?}] "));
            } else {
                ret.push_str(&format!("{player_selection:?} "));
            }
        }

        ret.push_str("\nPress 'Enter' to Start a New Game ");
        ret
    }

    fn on_tick(&mut self) {
        if self.menu_active {
            return;
        }
        if self.current_actions.is_empty() {
            return;
        }

        if let Some(state) = &mut self.state {
            if self.current_actions.len() == 1 {
                self.current_actions[0].0.apply_choice(state);
            } else {
                match state.player_type(state.current_player_idx) {
                    PlayerType::Human => {
                        if !self.move_selected {
                            return;
                        }
                        self.current_actions[self.selection_y].0.apply_choice(state);
                        self.move_selected = false;
                    }
                    PlayerType::MctsAI | PlayerType::TdAI => {
                        let ai = self.ai[state.current_player_idx].as_mut().unwrap();
                        if ai.num_games_sampled == 0 {
                            self.records = AI::get_simulation_records(state);
                        }

                        if !self.move_selected && ai.num_games_sampled < NUM_GAMES_TO_SIMULATE {
                            ai.sample_once(&mut self.records, state, DEPTH);
                            AI::sort_records(&mut self.records);
                            return;
                        }

                        self.records[0].action.apply_choice(state);
                        ai.reset();
                        self.move_selected = false;
                    }
                }
            }
            self.current_actions = Action::next_choices(state);
            self.num_selections_y = self.current_actions.len();
            self.selection_y = 0;
        }
    }

    fn start_new_game(&mut self) {
        let mut players = Vec::new();
        for (i, player_selection) in self.player_selections.iter().enumerate() {
            match player_selection {
                PlayerSelection::Human => players.push(PlayerType::Human),
                PlayerSelection::MctsAI => {
                    players.push(PlayerType::MctsAI);
                    self.ai[i] = Some(AI::new());
                }
                PlayerSelection::TdAI => {
                    players.push(PlayerType::TdAI);
                    self.ai[i] = Some(AI::new());
                }
                PlayerSelection::Empty => (),
            }
        }
        self.state = State::new(&players);
        if let Some(state) = &self.state {
            if self.menu_active {
                self.menu_active = false;
            }

            self.current_actions = Action::next_choices(state);
        }
    }

    pub fn format_next_actions(&self) -> String {
        if self.current_actions.is_empty() {
            return "GAME OVER!".to_string();
        }

        let mut ret: String = String::new();
        let mut additional_stuff: String = String::new();
        if let Some(state) = &self.state {
            match state.player_type(state.current_player_idx) {
                PlayerType::Human => {
                    for (i, (action, _)) in self.current_actions.iter().enumerate() {
                        if i == self.selection_y {
                            ret.push_str(&format!("\n>> {action:?}"));
                            if let Action::Fence(pc) = action {
                                let mut farm = state.current_farm().clone();
                                let mut wood = state.current_player_quantities()[Wood.index()];
                                farm.fence_spaces(pc, &mut wood);
                                let room_material_index =
                                    state.room_material_idx(state.current_player_idx);
                                additional_stuff = print_farm(&farm, room_material_index);
                            }
                        } else {
                            ret.push_str(&format!("\n{action:?}"));
                        }
                    }
                }
                PlayerType::MctsAI | PlayerType::TdAI => {
                    let ai = self.ai[state.current_player_idx].as_ref().unwrap();
                    ret = format!(
                        "{}/{} Games Simulated..\nCache Size {}\n",
                        ai.num_games_sampled,
                        NUM_GAMES_TO_SIMULATE,
                        ai.cache.len()
                    );

                    for (i, rec) in self.records.iter().enumerate() {
                        if i == 0 {
                            ret.push_str(&format!(
                                "\n>> [{:.2} / {}] {:?}",
                                rec.score, rec.games, rec.action
                            ));

                            if let Action::Fence(_) = &rec.action {
                                additional_stuff = String::from("Fence Layout : TODO");
                            }
                        } else {
                            ret.push_str(&format!(
                                "\n[{:.2} / {}] {:?}",
                                rec.score, rec.games, rec.action
                            ));
                        }
                    }
                }
            }

            ret = format!("{ret}\n\n\n{additional_stuff}");
        }
        ret
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUN_BACKTRACE", "1");
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(1);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Down => {
                        app.next_y();
                    }
                    KeyCode::Up => {
                        app.previous_y();
                    }
                    KeyCode::Right => {
                        app.next_x();
                    }
                    KeyCode::Left => {
                        app.previous_x();
                    }
                    KeyCode::Char('n') => {
                        app.menu_active = !app.menu_active;
                        if app.menu_active {
                            app.num_selections_y = PLAYER_TYPES.len();
                            app.selection_y = 0;
                        }
                    }
                    KeyCode::Enter => {
                        if app.menu_active {
                            app.start_new_game();
                        } else {
                            app.move_selected = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Action Spaces
    let block1 = Block::default()
        .title("Board")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let block2 = Block::default()
        .title("Farms")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let block3 = Block::default()
        .title("Available Actions")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    if let Some(state) = &app.state {
        // Board State
        let text = Paragraph::new(state.format())
            .block(block1)
            .style(Style::default());
        f.render_widget(text, chunks[0]);

        // Player farms
        let n = u16::try_from(state.num_players).unwrap();
        let padding: u16 = 2;
        let board_size = (100 - padding * (n - 1)) / n;
        let mut constraints = vec![Constraint::Percentage(board_size)];
        for _ in 1..n {
            constraints.push(Constraint::Percentage(padding));
            constraints.push(Constraint::Percentage(board_size));
        }
        let farm_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(chunks[1]);
        for i in 0..state.num_players {
            let scores = state.scores();
            let mut title_string = format!(
                " Player {} | {:?} | {} Points",
                i + 1,
                state.player_type(i),
                scores[i]
            );

            if i == state.starting_player_idx {
                title_string = format!("{title_string} | 🟡 ");
            }

            if i == state.current_player_idx {
                title_string = format!("{title_string} | 🔻 ");
            }

            let farm = Block::default().title(title_string).borders(Borders::ALL);

            f.render_widget(farm, farm_areas[2 * i]);

            let displays = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(45),
                        Constraint::Percentage(10),
                        Constraint::Percentage(45),
                    ]
                    .as_ref(),
                )
                .split(farm_areas[2 * i]);

            let farm_strings = print_farm(state.player_farm(i), state.room_material_idx(i));
            let main_farm = Paragraph::new(farm_strings.to_string())
                .style(Style::default())
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(main_farm, displays[0]);

            let resource_text = Paragraph::new(print_resources(state, i));
            f.render_widget(resource_text, displays[2]);
        }

        // Actions
        let text = Paragraph::new(app.format_next_actions())
            .block(block3)
            .style(Style::default());
        f.render_widget(text, chunks[2]);
    } else {
        let text = Paragraph::new("Press 'N' to set up a new game")
            .block(block1)
            .style(Style::default());
        f.render_widget(text, chunks[0]);
        f.render_widget(block2, chunks[1]);
        f.render_widget(block3, chunks[2]);
    }

    if app.menu_active {
        let block = Block::default()
            .title("Create New Game")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let area = centered_rect(80, 40, f.size());
        f.render_widget(Clear, area); //this clears out the background

        let text = Paragraph::new(app.show_menu().to_string()).block(block);
        f.render_widget(text, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
