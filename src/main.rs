use std::time::{Duration, Instant};
use std::{error::Error, io};

mod agricola;

use agricola::scoring;
use agricola::state::State;
use agricola::{actions::Action, algorithms::PlayerType};
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

#[derive(Clone, Copy, Debug)]
enum PlayerSelection {
    Empty,
    Human,
    RandomAI,
    UniformAI,
    MctsAI,
}

const PLAYER_TYPES: [PlayerSelection; 5] = [
    PlayerSelection::Empty,
    PlayerSelection::Human,
    PlayerSelection::RandomAI,
    PlayerSelection::UniformAI,
    PlayerSelection::MctsAI,
];

struct App {
    selection_x: usize,
    selection_y: usize,
    player_selections: [PlayerSelection; 4],
    state: Option<State>,
    menu_active: bool,
    num_selections_y: usize,
    game_over: bool,
    human_chose: bool,
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
            game_over: false,
            human_chose: false,
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
                ret = format!("{ret}[{:?}] ", player_selection);
            } else {
                ret = format!("{ret}{:?} ", player_selection);
            }
        }

        ret = format!("{ret}\nPress 'Enter' to Start a New Game ");
        ret
    }

    fn on_tick(&mut self) {
        let chosen_action = self.play();
        self.human_chose = false;
        if chosen_action.is_some() {
            if let Some(action) = &chosen_action {
                if let Some(state) = &mut self.state {
                    action.apply_choice(state);
                }
            }
        }
    }

    fn start_new_game(&mut self) {
        let mut players = Vec::new();
        for player_selection in self.player_selections {
            match player_selection {
                PlayerSelection::Human => players.push(PlayerType::Human),
                PlayerSelection::MctsAI => players.push(PlayerType::MCTSMachine),
                PlayerSelection::RandomAI => players.push(PlayerType::RandomMachine),
                PlayerSelection::UniformAI => players.push(PlayerType::UniformMachine),
                _ => (),
            }
        }
        self.state = State::new(&players);
        if self.state.is_some() && self.menu_active {
            self.menu_active = false;
        }
    }

    pub fn format_next_actions(&self) -> String {
        let mut ret: String = String::new();
        if let Some(state) = &self.state {
            let actions = Action::next_choices(state);
            if actions.is_empty() {
                ret = "GAME OVER!".to_string();
            } else {
                for (i, action) in actions.iter().enumerate() {
                    if i == self.selection_y && matches!(state.player_type(), PlayerType::Human) {
                        ret = format!("{}\n>> {:?}", ret, action);
                    } else {
                        ret = format!("{}\n{:?}", ret, action);
                    }
                }
            }
        }
        ret
    }

    fn play(&mut self) -> Option<Action> {
        if let Some(state) = &mut self.state {
            let actions = Action::next_choices(state);
            if !actions.is_empty() {
                if actions.len() == 1 {
                    return Some(actions[0].clone());
                } else {
                    let action = match state.player_type() {
                        PlayerType::Human => {
                            self.num_selections_y = Action::next_choices(state).len();
                            if !self.human_chose {
                                return None;
                            }

                            if self.selection_y < self.num_selections_y {
                                Some(actions[self.selection_y].clone())
                            } else {
                                None
                            }
                        }
                        _ => state.player_type().best_action(state, false),
                    };

                    return action;
                }
            }
            self.game_over = true;
        }
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
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
                        }
                    }
                    KeyCode::Enter => {
                        if app.menu_active {
                            app.start_new_game();
                        } else {
                            app.human_chose = true;
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
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
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
        let n = state.players.len() as u16;
        let padding: u16 = 2;
        let board_size = (100 - padding * (n - 1)) / n;
        let mut constraints = vec![Constraint::Percentage(board_size)];
        for _ in 1..n {
            constraints.push(Constraint::Percentage(padding));
            constraints.push(Constraint::Percentage(board_size));
        }
        let farm_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints.as_ref())
            .split(chunks[1]);
        for (i, p) in state.players.iter().enumerate() {
            let mut title_string = format!(
                " Player {} | {:?} | {} Points",
                i + 1,
                p.player_type(),
                scoring::score(p)
            );

            if i == state.starting_player_idx {
                title_string = format!("{title_string} | Starting Player ");
            }

            if i == state.current_player_idx {
                title_string = format!("{title_string} | Turn ");
            }

            let farm = Block::default().title(title_string).borders(Borders::ALL);

            f.render_widget(farm, farm_areas[2 * i]);

            let displays = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(2),
                        Constraint::Percentage(68),
                        Constraint::Percentage(2),
                        Constraint::Percentage(28),
                    ]
                    .as_ref(),
                )
                .split(farm_areas[2 * i]);

            let farm_display = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(70),
                    Constraint::Percentage(10),
                ])
                .split(displays[1]);

            let text = Paragraph::new(p.display_farm().to_string()).style(Style::default());
            f.render_widget(text, farm_display[1]);

            let resource_text = Paragraph::new(p.display_resources());
            let resource_display = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(33); 3])
                .split(displays[3]);
            f.render_widget(resource_text, resource_display[1]);
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
