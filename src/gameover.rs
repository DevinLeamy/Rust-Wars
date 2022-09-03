use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{GameState, aliens::Alien, player::Ship, shared::Bullet};

#[derive(Component)]
struct GameOverMenu;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::GameOver, create_gameover_screen)
            .add_exit_system(GameState::GameOver, despawn_entities)
            .add_system(update_gameover_menu.run_in_state(GameState::GameOver));
    }
}

fn create_gameover_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    alien_query: Query<Entity, With<Alien>> // aliens should be cleared by the AliensPlugin
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
                    "[R] To Retry\n[ESC] To Quit",
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


fn update_gameover_menu(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>, 
) {
    let restart = keyboard_input.pressed(KeyCode::R);

    if restart {
        commands.insert_resource(NextState(GameState::Playing));
    }
}

fn despawn_entities(
    mut commands: Commands,
    ship_query: Query<Entity, With<Ship>>,
    bullet_query: Query<Entity, With<Bullet>>,
    menu_query: Query<Entity, With<GameOverMenu>>
) {
    let ship = ship_query.single();
    commands.entity(ship).despawn();

    for bullet in bullet_query.iter() {
        commands.entity(bullet).despawn();
    }

    let gameover_menu = menu_query.single();
    commands.entity(gameover_menu).despawn();

}