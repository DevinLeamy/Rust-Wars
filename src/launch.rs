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
                            font_size: 110.0,
                            color: Color::rgb(1.0, 1.0, 1.0),
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        },
                    ),
                    TextSection::new(
                        "Chapter II: The Borrow Checker's Return\n\n\n",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::rgb(0.7, 0.7, 0.7),
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        },
                    ),
                    TextSection::new(
                        "→ Press [space] to play ←",
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
        commands
            .spawn()
            .insert(Menu)
            .insert(Name::new("Menu Ferris"))
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(-186.0, -290.0, 0.0),
                    scale: Vec3::new(1.3, 1.3, 1.0),
                    rotation: Quat::from_rotation_z(-0.2),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(300.0, 200.0)),
                    ..default()
                },
                texture: asset_server.load("images/ferris.png"),
                ..default()
            });

        commands
            .spawn()
            .insert(Menu)
            .insert(Name::new("Menu Alien Ferris"))
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(188.0, 270.0, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                    rotation: Quat::from_rotation_z(0.0f32.to_radians()),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(300.0, 200.0)),
                    ..default()
                },
                texture: asset_server.load("images/alien_ferris.png"),
                ..default()
            });

        commands
            .spawn()
            .insert(Menu)
            .insert(Name::new("Menu Unsafe Ferris"))
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(-304.0, 294.0, 0.0),
                    scale: Vec3::new(0.7, 0.7, 1.0),
                    rotation: Quat::from_rotation_z(0.2),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(150.0, 150.0)),
                    ..default()
                },
                texture: asset_server.load("images/unsafe_ferris_2.png"),
                ..default()
            });
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
        for menu_entity in query.iter() {
            commands.entity(menu_entity).despawn();
        }
    }
}

