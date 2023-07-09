use agricola::actions::Action;
use agricola::algorithms::PlayerType;
use agricola::farm::{FarmyardSpace, House, MASK};
use agricola::player::Player;
use agricola::primitives;
use agricola::state::State;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::egui::Pos2;
use bevy_egui::EguiPlugin;
use rand::seq::SliceRandom;

use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};

// use std::env;
// use std::time::Instant;

mod agricola;

const LX: f32 = 1000.0;
const LY: f32 = 600.0;
const PADDING: f32 = 10.0;
const FARM_LEN: f32 = 400.0;
const FARM_WID: f32 = 240.0;

#[derive(Component)]
struct ActionSpaceTile;

#[derive(Resource)]
struct ContinuousPlay(Timer);

#[derive(Resource)]
struct Game {
    num_players: usize,
    player_type: crate::agricola::algorithms::PlayerType,
    state: crate::agricola::state::State,
    colors: [Color; 4],
}

impl Default for Game {
    fn default() -> Self {
        Self {
            num_players: 1,
            player_type: agricola::algorithms::PlayerType::RandomMachine,
            state: agricola::state::State::create_new(
                1,
                false,
                &agricola::algorithms::PlayerType::RandomMachine,
            ),
            colors: [Color::RED, Color::DARK_GREEN, Color::PURPLE, Color::CYAN],
        }
    }
}

struct PlayMoveEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Agricola".into(),
                resolution: WindowResolution::new(2.0 * LX, 2.0 * LY),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ContinuousPlay(Timer::from_seconds(0.5, TimerMode::Once)))
        .add_event::<PlayMoveEvent>()
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<Game>()
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(ui_system)
        .add_system(play)
        .add_system(play_continuous)
        .add_system(button_system)
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn(Camera2dBundle::default());
}

const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct ClickableAction(Action);

fn button_system(
    mut game: ResMut<Game>,
    //mut new_game_event_writer: EventWriter<PlayMoveEvent>,
    mut interaction_query: Query<
        (&Interaction, &ClickableAction),
        (Changed<Interaction>, With<Button>, With<ClickableAction>),
    >,
) {
    for (interaction, caction) in &mut interaction_query {
        //println!("Interaction Captured");
        match *interaction {
            Interaction::Clicked => {
                println!("Chosen Action {:?}", caction.0);
                caction.0.apply_choice(&mut game.state);
                //new_game_event_writer.send(PlayMoveEvent);
            }
            _ => (),
        }
    }
}

fn draw_boxes_with_text(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    y_level: f32,
    text_sections: &Vec<(Vec<String>, Color, Option<Action>)>,
    font_size: f32,
) -> f32 {
    let font = asset_server.load("fonts/AmericanTypewriterRegular.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size,
        color: Color::WHITE,
    };

    let mut box_position = Vec2::new(PADDING, y_level + PADDING);

    //draw_box(commands, &Color::RED, &Vec2 { x: 2.0 * LX, y: LY }, &Vec2 { x: 0.0, y: LY });

    let w = text_sections.iter().map(|s| s.0.len()).max().unwrap_or(0) as f32;
    let padding = font_size;
    let l_scale = 0.7;
    let w_scale = 1.5;

    for (texts, color, opt_action) in text_sections {
        let l = texts.iter().map(|s| s.len()).max().unwrap_or(0) as f32;
        let box_size = Vec2::new(font_size * l * l_scale, font_size * w * w_scale);

        if box_position.x + box_size.x + padding > 2.0 * LX - PADDING {
            box_position.x = PADDING;
            box_position.y += box_size.y + padding;
        }

        draw_box_with_text(
            commands,
            &color,
            &box_size,
            &box_position,
            texts,
            &text_style,
            opt_action.clone(),
        );
        box_position.x += box_size.x + padding;
    }

    box_position.y
}

fn draw_farm(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player: &Player,
    color: Color,
    center: &Vec2,
    farmyard_space_size: f32,
    border_size: f32,
) {
    const SX: usize = 5;
    const SY: usize = 3;
    let l: f32 = SX as f32 * farmyard_space_size + (SX as f32 + 1.0) * border_size;
    let w: f32 = SY as f32 * farmyard_space_size + (SY as f32 + 1.0) * border_size;

    let mut box_position = Vec2::new(center.x - l / 2.0, center.y + w / 2.0);
    let mut box_size = Vec2::new(farmyard_space_size, farmyard_space_size);

    let font = asset_server.load("fonts/AmericanTypewriterRegular.ttf");

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 10.0,
        color: Color::BLACK,
    };

    // populate fence masks
    let mut fence_mask: Vec<Vec<bool>> = vec![vec![false; 2 * SY + 1]; 2 * SX + 1];
    for r in 0..SY {
        let y1 = 2 * r;
        let y = 2 * r + 1;
        let y2 = 2 * r + 2;

        for c in 0..SX {
            let idx = r * 5 + c;
            let x1 = 2 * c;
            let x = 2 * c + 1;
            let x2 = 2 * c + 2;

            match player.farm.farmyard_spaces[idx] {
                FarmyardSpace::FencedPasture(mask, _, _) => {
                    if mask & MASK[0] > 0 {
                        fence_mask[x][y1] = true;
                    }
                    if mask & MASK[1] > 0 {
                        fence_mask[x2][y] = true;
                    }
                    if mask & MASK[2] > 0 {
                        fence_mask[x1][y] = true;
                    }
                    if mask & MASK[3] > 0 {
                        fence_mask[x][y2] = true;
                    }
                }
                _ => (),
            }
        }
    }

    for ii in 0..2 * SY + 1 {
        if ii % 2 == 0 {
            box_size.y = border_size;
        } else {
            box_size.y = farmyard_space_size;
        }
        for jj in 0..2 * SX + 1 {
            if jj % 2 == 0 {
                box_size.x = border_size;
            } else {
                box_size.x = farmyard_space_size;
            }

            let mut farmyard_space_pos = box_position.clone();
            farmyard_space_pos.x += box_size.x / 2.0;
            farmyard_space_pos.y -= box_size.y / 2.0;

            if (ii + jj) % 2 == 1 {
                // Fence
                if fence_mask[jj][ii] {
                    draw_box(commands, &color, &box_size, &farmyard_space_pos);
                }
            } else if ii % 2 == 1 && jj % 2 == 1 {
                // Farmyard space
                let idx = (ii / 2) * 5 + jj / 2;
                let mut text_sections: Vec<String> = vec![];

                match player.farm.farmyard_spaces[idx] {
                    FarmyardSpace::Empty => {
                        draw_box(
                            commands,
                            &Color::YELLOW_GREEN,
                            &box_size,
                            &farmyard_space_pos,
                        );
                    }
                    FarmyardSpace::Room => match player.house {
                        House::Wood => {
                            draw_box(commands, &Color::OLIVE, &box_size, &farmyard_space_pos)
                        }
                        House::Clay => {
                            draw_box(commands, &Color::TOMATO, &box_size, &farmyard_space_pos)
                        }
                        House::Stone => {
                            draw_box(commands, &Color::DARK_GRAY, &box_size, &farmyard_space_pos)
                        }
                    },
                    FarmyardSpace::Field(opt_seed) => match opt_seed {
                        Some((seed, amt)) => {
                            text_sections.push(format!("{} {:?}", amt, seed));
                            draw_box_with_text(
                                commands,
                                &Color::DARK_GREEN,
                                &box_size,
                                &farmyard_space_pos,
                                &text_sections,
                                &text_style,
                                None,
                            );
                        }
                        None => {
                            draw_box(commands, &Color::DARK_GREEN, &box_size, &farmyard_space_pos)
                        }
                    },
                    FarmyardSpace::UnfencedStable(opt_animal) => match opt_animal {
                        Some((animal, amt)) => {
                            text_sections.push(format!("{} {:?}", amt, animal));
                            draw_box_with_text(
                                commands,
                                &Color::YELLOW_GREEN,
                                &box_size,
                                &farmyard_space_pos,
                                &text_sections,
                                &text_style,
                                None,
                            );

                            draw_box(
                                commands,
                                &Color::YELLOW,
                                &Vec2 {
                                    x: box_size.x / 10.0,
                                    y: box_size.y / 10.0,
                                },
                                &Vec2 {
                                    x: farmyard_space_pos.x + box_size.x * 0.4,
                                    y: farmyard_space_pos.y + box_size.y * 0.4,
                                },
                            );
                        }
                        None => draw_box(
                            commands,
                            &Color::YELLOW_GREEN,
                            &box_size,
                            &farmyard_space_pos,
                        ),
                    },
                    FarmyardSpace::FencedPasture(_, opt_animal, has_stable) => {
                        match opt_animal {
                            Some((animal, amt)) => {
                                text_sections.push(format!("{} {:?}", amt, animal));
                                draw_box_with_text(
                                    commands,
                                    &Color::YELLOW_GREEN,
                                    &box_size,
                                    &farmyard_space_pos,
                                    &text_sections,
                                    &text_style,
                                    None,
                                );
                            }
                            None => draw_box(
                                commands,
                                &Color::YELLOW_GREEN,
                                &box_size,
                                &farmyard_space_pos,
                            ),
                        }

                        if has_stable {
                            draw_box(
                                commands,
                                &Color::YELLOW,
                                &Vec2 {
                                    x: box_size.x / 10.0,
                                    y: box_size.y / 10.0,
                                },
                                &Vec2 {
                                    x: farmyard_space_pos.x + box_size.x * 0.4,
                                    y: farmyard_space_pos.y + box_size.y * 0.4,
                                },
                            );
                        }
                    }
                }
            }

            box_position.x += box_size.x;
        }
        box_position.y -= box_size.y;
        box_position.x = center.x - l / 2.0;
    }
}

fn draw_sprite(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    color: Color,
    path: &str,
    position: &Vec2,
    box_size: &Vec2,
    rot: Quat,
) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                ..default()
            },
            texture: asset_server.load(path),
            transform: Transform {
                translation: position.extend(0.0),
                rotation: rot,
                ..default()
            },
            ..default()
        },
        ActionSpaceTile,
    ));
}

fn draw_box(commands: &mut Commands, box_color: &Color, box_size: &Vec2, box_position: &Vec2) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: *box_color,
                custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                ..default()
            },
            transform: Transform::from_translation(box_position.extend(0.0)),
            ..default()
        },
        ActionSpaceTile,
    ));
}

fn draw_box_with_text(
    commands: &mut Commands,
    box_color: &Color,
    box_size: &Vec2,
    box_position: &Vec2,
    text: &Vec<String>,
    text_style: &TextStyle,
    opt_action: Option<Action>,
) {
    let text_sections: Vec<TextSection> = text
        .iter()
        .map(|s| TextSection::new(s, text_style.clone()))
        .collect();

    let e_id = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position: UiRect {
                        left: Val::Px(box_position.x),
                        top: Val::Px(box_position.y),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Px(box_size.x), Val::Px(box_size.y)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(*box_color),
                transform: Transform::from_translation(box_position.extend(0.0)),
                ..default()
            },
            ActionSpaceTile,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections(text_sections));
        })
        .id();

    if let Some(action) = opt_action {
        commands.entity(e_id).insert(ClickableAction(action));
    }
}

fn play_continuous(
    time: Res<Time>,
    mut timer: ResMut<ContinuousPlay>,
    mut new_game_event_writer: EventWriter<PlayMoveEvent>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        new_game_event_writer.send(PlayMoveEvent);
    }
}

fn play(
    mut game: ResMut<Game>,
    mut commands: Commands,
    query: Query<Entity, With<ActionSpaceTile>>,
    mut new_game_event_reader: EventReader<PlayMoveEvent>,
    asset_server: Res<AssetServer>,
) {
    for _ in new_game_event_reader.iter() {
        // Remove old tiles
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let font_size = 16.0;
        let mut cursor = Vec2::new(0.0, 100.0);

        let mut texts: Vec<(Vec<String>, Color, Option<Action>)> = Vec::new();
        let mut color: Color;

        // Draw Action Spaces
        for action in &game.state.open_spaces {
            let idx = action.action_idx();
            let mut res_str = String::new();

            if action.resource_map_idx().is_some() {
                res_str = primitives::format_resources(
                    &game.state.resource_map[action.resource_map_idx().unwrap()],
                );
            }

            if game.state.occupied_spaces.contains(&idx) {
                color = Color::GRAY;
            } else {
                color = Color::SEA_GREEN;
            }
            texts.push((
                vec![format!("{:?}\n", &action), format!("{}", &res_str)],
                color,
                None,
            ));
        }

        cursor.y = draw_boxes_with_text(&mut commands, &asset_server, cursor.y, &texts, font_size);

        texts.clear();
        cursor.y += 100.0;

        // Majors on Board
        for major in &game.state.major_improvements {
            texts.push((vec![format!("{:?}\n", &major)], Color::MAROON, None));
        }

        cursor.y = draw_boxes_with_text(&mut commands, &asset_server, cursor.y, &texts, font_size);

        cursor.y += 2.0 * PADDING;
        cursor.x = PADDING;

        // Display available moves
        let mut available_actions = Action::next_choices(&game.state);

        let font = asset_server.load("fonts/AmericanTypewriterRegular.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 20.0,
            color: Color::WHITE,
        };

        texts.clear();
        if available_actions.is_empty() {
            texts.push((vec![format!("GAME OVER!")], Color::BLACK, None));
        } else {
            while available_actions.len() == 1 {
                available_actions[0].apply_choice(&mut game.state);
                available_actions = Action::next_choices(&game.state);
            }

            match game.state.players[game.state.current_player_idx].player_type {
                PlayerType::MCTSMachine
                | PlayerType::UniformMachine
                | PlayerType::RandomMachine => {
                    let opt_action = game.player_type.best_action(&game.state, false);
                    if let Some(action) = opt_action {
                        for action in &available_actions {
                            texts.push((
                                vec![format!("{:?}\n", action)],
                                Color::MIDNIGHT_BLUE,
                                None,
                            ));
                        }

                        cursor.y = draw_boxes_with_text(
                            &mut commands,
                            &asset_server,
                            cursor.y,
                            &texts,
                            font_size,
                        );
                        draw_box_with_text(
                            &mut commands,
                            &Color::BLACK,
                            &Vec2 { x: 200.0, y: 50.0 },
                            &Vec2 {
                                x: LX,
                                y: cursor.y + 50.0,
                            },
                            &vec![format!("Chosen Action {:?}", action)],
                            &text_style,
                            None,
                        );
                        action.apply_choice(&mut game.state);
                    }
                }
                PlayerType::Human => {
                    for action in &available_actions {
                        texts.push((
                            vec![format!("{:?}\n", action)],
                            Color::MIDNIGHT_BLUE,
                            Some(action.clone()),
                        ));
                    }

                    cursor.y = draw_boxes_with_text(
                        &mut commands,
                        &asset_server,
                        cursor.y,
                        &texts,
                        font_size,
                    );
                }
            }
        }

        // Players
        let n = game.state.players.len();
        let farmyard_space_padding = 10.0;
        let offset_x = (2.0 * LX - FARM_LEN * n as f32) / (n + 1) as f32;
        let farmyard_space_size = (FARM_LEN - (n - 1) as f32 * farmyard_space_padding) / 5.0;
        let box_size = Vec2::new(farmyard_space_size, farmyard_space_size);

        cursor.x = -LX + 300.0;
        cursor.y = LY - cursor.y - 300.0;

        for (i, p) in game.state.players.iter().enumerate() {
            let start_player_meeple_position =
                Vec2::new(cursor.x - FARM_LEN / 2.0, cursor.y + FARM_WID / 2.0 + 70.0);
            let turn_signal_position = Vec2::new(cursor.x, cursor.y + FARM_WID / 2.0 + 70.0);

            if i == game.state.starting_player_idx {
                draw_sprite(
                    &mut commands,
                    &asset_server,
                    Color::YELLOW,
                    "img/start_player.png",
                    &start_player_meeple_position,
                    &box_size,
                    Quat::IDENTITY,
                );
            }

            if i == game.state.current_player_idx {
                draw_sprite(
                    &mut commands,
                    &asset_server,
                    game.colors[i],
                    "img/turn.png",
                    &turn_signal_position,
                    &box_size,
                    Quat::IDENTITY,
                );
            }

            draw_farm(
                &mut commands,
                &asset_server,
                p,
                game.colors[i],
                &cursor,
                farmyard_space_size,
                farmyard_space_padding,
            );

            cursor.x += FARM_LEN + offset_x;
        }
    }
}

fn ui_system(
    mut game: ResMut<Game>,
    mut new_game_event_writer: EventWriter<PlayMoveEvent>,
    mut contexts: EguiContexts,
    mut timer: ResMut<ContinuousPlay>,
) {
    egui::Window::new("Agricola")
        .default_pos(Pos2::new(2.0 * LX, 2.0 * LY))
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Number of Players: ");
                ui.add(Slider::new(&mut game.num_players, 1..=4));
            });

            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{:?}", game.player_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut game.player_type,
                        PlayerType::RandomMachine,
                        "Random AI",
                    );
                    ui.selectable_value(
                        &mut game.player_type,
                        PlayerType::UniformMachine,
                        "Uniform AI",
                    );
                    ui.selectable_value(&mut game.player_type, PlayerType::MCTSMachine, "MCTS AI");
                });

            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Start New Game")).clicked() {
                    game.state = State::create_new(game.num_players, true, &game.player_type);
                    game.colors.shuffle(&mut rand::thread_rng());
                    new_game_event_writer.send(PlayMoveEvent);
                }
            });

            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Play Move")).clicked() {
                    new_game_event_writer.send(PlayMoveEvent);
                }
            });

            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("Toggle Continuous Play"))
                    .clicked()
                {
                    if timer.0.mode() == TimerMode::Once {
                        timer.0.set_mode(TimerMode::Repeating);
                    } else {
                        timer.0.set_mode(TimerMode::Once);
                    }
                }
            })
        });
}
