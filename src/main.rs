use agricola::algorithms::Kind;
use agricola::farm::{FarmyardSpace, House};
use agricola::primitives;
use agricola::state::State;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};

// use std::env;
// use std::time::Instant;

mod agricola;

#[macro_use]
extern crate lazy_static;

const LX: f32 = 1000.0;
const LY: f32 = 500.0;
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
    ai_kind: crate::agricola::algorithms::Kind,
    state: crate::agricola::state::State,
    running_status: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            num_players: 1,
            ai_kind: agricola::algorithms::Kind::RandomMachine,
            state: agricola::state::State::create_new(
                1,
                false,
                &agricola::algorithms::Kind::RandomMachine,
            ),
            running_status: true,
        }
    }
}

struct PlayMoveEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Agricola".into(),
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
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn(Camera2dBundle::default());
}

fn draw_sprite(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    color: Color,
    path: &str,
    position: &Vec2,
    box_size: &Vec2,
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
                ..default()
            },
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
    text: Vec<String>,
    text_style: &TextStyle,
) {
    let text_sections = text
        .iter()
        .map(|s| TextSection::new(s, text_style.clone()))
        .collect();

    commands
        .spawn((
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
        ))
        .with_children(|builder| {
            builder.spawn(Text2dBundle {
                text: Text {
                    sections: text_sections,
                    alignment: TextAlignment::Center,
                    ..default()
                },
                // ensure the text is drawn on top of the box
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            });
        });
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

        let font = asset_server.load("fonts/AmericanTypewriterRegular.ttf");

        let text_style = TextStyle {
            font: font.clone(),
            font_size: 16.0,
            color: Color::WHITE,
        };

        let mut box_size = Vec2::new(LX / 5.0, LX / 20.0);
        let mut curr_x = -LX;
        let mut curr_y = LY;

        // Draw Action Spaces
        for action in &game.state.open_spaces {
            let idx = action.action_idx();
            let mut res_str = String::new();

            if action.resource_map_idx().is_some() {
                res_str = primitives::format_resources(
                    &game.state.resource_map[action.resource_map_idx().unwrap()],
                );
                //println!("{}", &res_str);
            }

            let box_position = Vec2::new(curr_x, curr_y);
            curr_x += box_size.x + PADDING;
            if curr_x >= LX {
                curr_x = -LX;
                curr_y -= box_size.y + PADDING;
            }
            let mut tile_color = Color::DARK_GREEN;

            if game.state.occupied_spaces.contains(&idx) {
                tile_color = Color::GRAY;
            }

            draw_box_with_text(
                &mut commands,
                &tile_color,
                &box_size,
                &box_position,
                vec![format!("{:?}\n", &action), format!("{}", &res_str)],
                &text_style,
            );
        }

        // Majors on Board
        curr_x = -LX;
        curr_y -= LX / 5.0;

        for major in &game.state.major_improvements {
            let box_position = Vec2::new(curr_x, curr_y);
            curr_x += box_size.x + PADDING;

            draw_box_with_text(
                &mut commands,
                &Color::RED,
                &box_size,
                &box_position,
                vec![format!("{:?}\n", &major)],
                &text_style,
            );
        }

        // Players
        let n = game.state.players.len();
        let offset_x = (2.0 * LX - FARM_LEN * n as f32) / (n + 1) as f32;
        let farmyard_space_padding = 2.0;
        let farmyard_space_size = (FARM_LEN - (n - 1) as f32 * farmyard_space_padding) / 5.0;
        box_size = Vec2::new(farmyard_space_size, farmyard_space_size);

        curr_x = -LX + offset_x;
        curr_y -= 200.0;

        for (i, p) in game.state.players.iter().enumerate() {
            let start_player_meeple_position =
                Vec2::new(curr_x - FARM_LEN / 2.0, curr_y + FARM_WID / 2.0);
            let turn_signal_position = Vec2::new(curr_x, curr_y + FARM_WID / 2.0);

            if i == game.state.starting_player_idx {
                draw_sprite(
                    &mut commands,
                    &asset_server,
                    Color::YELLOW,
                    "img/start_player.png",
                    &start_player_meeple_position,
                    &box_size,
                );
            }

            if i == game.state.current_player_idx {
                draw_sprite(
                    &mut commands,
                    &asset_server,
                    Color::RED,
                    "img/turn.png",
                    &turn_signal_position,
                    &box_size,
                );
            }

            // draw_box_with_text(
            //     &mut commands,
            //     &Color::BLUE,
            //     &box_size,
            //     &start_player_meeple_position,
            //     vec![p.format()],
            //     &text_style,
            // );

            let mut space_x = start_player_meeple_position.x;
            let mut space_y = start_player_meeple_position.y - 100.0;
            for r in 0..3 {
                for c in 0..5 {
                    let idx = r * 5 + c;
                    let box_position = Vec2::new(space_x, space_y);
                    let mut space_type = "img/empty.png";
                    let mut color = Color::GREEN;

                    match p.farm.farmyard_spaces[idx] {
                        FarmyardSpace::Room => {
                            space_type = "img/room.png";
                            match p.house {
                                House::Wood => color = Color::OLIVE,
                                House::Clay => color = Color::MAROON,
                                House::Stone => color = Color::DARK_GRAY,
                            }
                        }
                        FarmyardSpace::Field => {
                            space_type = "img/field.png";
                        }
                        FarmyardSpace::Pasture => {
                            space_type = "img/pasture.png";
                        }
                        _ => (),
                    }

                    draw_sprite(
                        &mut commands,
                        &asset_server,
                        color,
                        space_type,
                        &box_position,
                        &box_size,
                    );

                    space_x += farmyard_space_size + farmyard_space_padding;
                }
                space_x = start_player_meeple_position.x;
                space_y -= farmyard_space_size + farmyard_space_padding;
            }

            curr_x += FARM_LEN + offset_x;
        }

        if game.running_status {
            game.running_status = game.state.play_move(true);
        }
    }
}

fn ui_system(
    mut game: ResMut<Game>,
    mut new_game_event_writer: EventWriter<PlayMoveEvent>,
    mut contexts: EguiContexts,
    mut timer: ResMut<ContinuousPlay>,
) {
    egui::Window::new("Agricola").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Number of Players: ");
            ui.add(Slider::new(&mut game.num_players, 1..=4));
        });

        egui::ComboBox::from_label("Select one!")
            .selected_text(format!("{:?}", game.ai_kind))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut game.ai_kind, Kind::RandomMachine, "Random AI");
                ui.selectable_value(&mut game.ai_kind, Kind::UniformMachine, "Uniform AI");
                ui.selectable_value(&mut game.ai_kind, Kind::MCTSMachine, "MCTS AI");
            });

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Start New Game")).clicked() {
                game.state = State::create_new(game.num_players, false, &game.ai_kind);
                game.running_status = true;
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
