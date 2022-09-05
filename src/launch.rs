use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{GameState, Global};

#[derive(Component)]
struct Menu;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::Menu, Menu::initialize)
            .add_system(Menu::update.run_in_state(GameState::Menu))
            .add_exit_system(GameState::Menu, Menu::cleanup);
    }
}

impl Menu {
    fn initialize(
        mut commands: Commands,
        asset_server: Res<AssetServer>
    ) {
        commands
            .spawn()
            .insert(Menu)
            .insert(Name::new("Menu"))
            .insert_bundle(
                TextBundle::from_sections([
                    TextSection::new(
                        "Rust Wars\n",
                        TextStyle {
                            font_size: 70.0,
                            color: Color::rgb(1.0, 1.0, 1.0),
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        },
                    ),
                    TextSection::new(
                        "Chapter II: The Borrow Checker's Return\n\n\n",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::rgb(0.8, 0.8, 0.8),
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        },
                    ),
                    TextSection::new(
                        "Press [space] to play",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(1.0, 1.0, 1.0),
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        },
                    ),
                ])
                .with_style(Style {
                    position_type: PositionType::Relative,
                    margin: UiRect {
                        top: Val::Auto,
                        left: Val::Auto,
                        right: Val::Auto,
                        bottom: Val::Auto,
                    },
                    align_self: AlignSelf::Center,
                    ..default()
                }),
            );
    } 

    fn update(
        mut commands: Commands,
        keyboard_input: Res<Input<KeyCode>>,
        mut global: ResMut<Global>,
    ) {
        let play = keyboard_input.just_pressed(KeyCode::Space);

        if play {
            global.start_playing();
            commands.insert_resource(NextState(GameState::LoadWaveState));
        }
    }

    fn cleanup(
        mut commands: Commands,
        query: Query<Entity, With<Menu>>
    ) {
        let menu = query.single();
        commands.entity(menu).despawn();
    }
}

