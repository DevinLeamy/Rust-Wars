use benimator::*;
use bevy::prelude::*;
use std::time::Duration;

use crate::config::*; 
use crate::utils::{Direction, PlayerState, Player, Velocity, Acceleration, MouseLocation};
use crate::animations::{Animation, Animations};

pub const PLAYER_SPEED: f32 = 70.0;
pub const PLAYER_JUMP_POWER: f32 = 20.0;

pub const PLAYER_WIDTH: f32 = 1.0;
pub const PLAYER_HEIGHT: f32 = 2.0;

pub const MAX_PLAYER_SPEED: f32 = 700.0;

pub const PLAYER_LAYER: f32 = 90.0;

fn compute_spawn_translation() -> Vec3 {
    Vec3::new(
        WORLD_ZERO_X + 2.0 * UNIT_WIDTH as f32,
        WORLD_ZERO_Y + 2.0 * UNIT_HEIGHT as f32,
        PLAYER_LAYER as f32,
    )
}

pub fn initialize_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    let player_scale = Vec3::new(
        PLAYER_WIDTH * UNIT_WIDTH as f32,
        PLAYER_HEIGHT * UNIT_HEIGHT as f32,
        1.0,
    );

    let spawn_translation = compute_spawn_translation();

    let mut player_animations = Animations::new();
    let walking_atlas = TextureAtlas::from_grid(
        asset_server.load("textures/character/animations/walk.png"),
        Vec2::new(56.0, 59.0),
        4, // columns
        6, // rows
    );
    let walking_ss_animation = SpriteSheetAnimation::from_range(0..=22, Duration::from_millis(30));

    let walking_animation = Animation::new(
        &texture_atlases.add(walking_atlas),
        &animations.add(walking_ss_animation),
    );

    let idle_atlas = TextureAtlas::from_grid(
        asset_server.load("textures/character/animations/idle.png"),
        Vec2::new(60.0, 58.0),
        4, // columns
        4, // rows
    );
    let idle_ss_animation = SpriteSheetAnimation::from_range(0..=13, Duration::from_millis(30));

    let idle_animation = Animation::new(
        &texture_atlases.add(idle_atlas),
        &animations.add(idle_ss_animation),
    );

    player_animations.add("WALK".to_string(), walking_animation);
    player_animations.add("IDLE".to_string(), idle_animation);

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: player_animations
                .get_handle("IDLE".to_string())
                .get_texture_atlas_handle(),
            transform: Transform {
                translation: spawn_translation,
                scale: Vec3::ONE, // player_scale,
                rotation: Quat::from_rotation_y(std::f32::consts::PI),
                ..default()
            },
            ..default()
        })
        .insert(
            player_animations
                .get_handle("IDLE".to_string())
                .get_animation_handle(),
        )
        .insert(Play)
        .insert(PlayerState::IDLE)
        .insert(Direction::RIGHT)
        .insert(Velocity(Vec2::new(0.0, 0.0)))
        .insert(Acceleration(Vec2::new(0.0, 0.0)))
        .insert(player_animations)
        .insert(Player);
}

pub fn move_player_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_position: Res<MouseLocation>,
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        &mut Acceleration,
        &mut Player,
        &mut Handle<SpriteSheetAnimation>,
        &mut Handle<TextureAtlas>,
        &mut Animations,
        &mut PlayerState,
        &mut Direction,
    )>,
) {
    let (
        mut player_transform,
        mut velocity,
        mut acceleration,
        mut player,
        mut animation,
        mut texture_atlas,
        animations,
        mut player_state,
        mut player_direction,
    ) = query.single_mut();

    let translation = player_transform.translation;

    let jump = keyboard_input.pressed(KeyCode::Space);
    let move_right = keyboard_input.pressed(KeyCode::D);
    let move_left = keyboard_input.pressed(KeyCode::A);

    let (new_direction, new_state) = if move_right {
        player_transform.translation.x += (UNIT_WIDTH as f32) / 20.0;
        (Direction::RIGHT, PlayerState::WALKING)
    } else if move_left {
        player_transform.translation.x -= (UNIT_WIDTH as f32) / 20.0;
        (Direction::LEFT, PlayerState::WALKING)
    } else {
        (player_direction.clone(), PlayerState::IDLE)
    };

    let update_direction = new_direction != *player_direction;
    let update_animation = new_state != *player_state;

    if update_direction {
        *player_direction = new_direction;
        match *player_direction {
            Direction::RIGHT => {
                player_transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
            }
            Direction::LEFT => {
                player_transform.rotation = Quat::default();
            }
            _ => (),
        }
    }

    if update_animation {
        *player_state = new_state;
        println!("UPDATE {:?}", player_state.clone());
        match *player_state {
            PlayerState::WALKING => {
                *texture_atlas = animations
                    .get_handle("WALK".to_string())
                    .get_texture_atlas_handle();
                *animation = animations
                    .get_handle("WALK".to_string())
                    .get_animation_handle();
            }
            PlayerState::IDLE => {
                *texture_atlas = animations
                    .get_handle("IDLE".to_string())
                    .get_texture_atlas_handle();
                *animation = animations
                    .get_handle("IDLE".to_string())
                    .get_animation_handle();
            }
            _ => (),
        }
    }
}

pub fn horizontally_center_player_system(
    player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_transform = player_query.single();
    let player_translation = player_transform.translation;

    let mut camera_transform = camera_query.single_mut();
    let camera_translation = &mut camera_transform.translation;

    camera_translation.x = player_translation.x;
}
