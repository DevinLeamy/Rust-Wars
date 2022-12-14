use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{aliens::Alien, shared::reset_game, GameState};

#[derive(Component)]
pub struct GameOverMenu;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameOver, create_gameover_screen)
            .add_exit_system(GameState::GameOver, reset_game)
            .add_system(update_gameover_menu.run_in_state(GameState::GameOver));
    }
}

fn create_gameover_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    alien_query: Query<Entity, With<Alien>>, // aliens should be cleared by the AliensPlugin
) {
    for alien_entity in alien_query.iter() {
        commands.entity(alien_entity).despawn();
    }

    commands
        .spawn()
        .insert(GameOverMenu)
        .insert_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "GAME OVER\n\n",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.8, 0.0, 0.0),
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                },
            ),
            TextSection::new(
                "[R] Retry\n[M] Menu\n[ESC] Quit",
                TextStyle {
                    font_size: 30.0,
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

fn update_gameover_menu(mut commands: Commands, keyboard_input: Res<Input<KeyCode>>) {
    let restart = keyboard_input.pressed(KeyCode::R);
    let menu = keyboard_input.pressed(KeyCode::M);

    if menu {
        commands.insert_resource(NextState(GameState::Menu));
    } else if restart {
        commands.insert_resource(NextState(GameState::LoadWaveState));
    }
}


