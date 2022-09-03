use bevy::prelude::*;
use rand::random;
use crate::{shared::*, aliens::ALIEN_SIZE, GameState};
use std::{time::Duration, collections::HashMap};
use iyes_loopless::prelude::*;

pub const SHIP_BULLET_IMAGE_SIZE: Vec2 = Vec2::new(512.0, 512.0);
const SHIP_IMAGE_SIZE: Vec2 = Vec2::new(1200.0, 800.0);
const SHIP_SIZE: Vec2 = Vec2::new(120., 80.);
const GAP_BETWEEN_SHIP_AND_FLOOR: f32 = 5.0;
const SHIP_SPEED: f32 = 450.;
const SHOOTING_COOLDOWN_IN_SECONDS: f32 = 0.8;
pub const SHIP_BULLET_SIZE: Vec2 = Vec2::new(33.0, 33.0);


#[derive(Component)]
pub struct Ship;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let mut fixedupdate = SystemStage::parallel();
        fixedupdate.add_system(update_ship.run_in_bevy_state(GameState::Playing));

        app
            .add_stage_before(
                CoreStage::Update,
                "Player_FixedUpdate",
                FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate)
            )
            .add_startup_system(spawn_player);
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
     // ship 
     let ship_y = BOTTOM_WALL + GAP_BETWEEN_SHIP_AND_FLOOR + SHIP_SIZE.y / 2.;

     commands
         .spawn()
         .insert(Ship)
         .insert_bundle(SpriteBundle {
             transform: Transform {
                 translation: Vec3::new(0.0, ship_y, 0.0),
                 scale: SHIP_SIZE.extend(1.0),
                 ..default()
             },
             sprite: generate_texture_sprite(ALIEN_SIZE, SHIP_IMAGE_SIZE), 
             texture: asset_server.load("images/ferris.png"),
             ..default()
         })
         .insert(Collider);
}

fn update_ship(
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<(Entity, &mut Transform, Option<&mut ShootingCooldown>), With<Ship>>, 
    sprites: Res<Sprites>,
    mut commands: Commands
) {
    let (ship, mut transform, mut shooting_cooldown) = query.single_mut(); 

    let mut direction = 0.;

    let move_left = keyboard_input.pressed(KeyCode::A);
    let move_right = keyboard_input.pressed(KeyCode::D);
    let shoot = keyboard_input.pressed(KeyCode::Space);

    if move_left {
        direction = -1.;
    } else if move_right {
        direction = 1.;
    }

    transform.translation.x += direction * SHIP_SPEED * TIME_STEP;

    transform.translation.x = transform.translation.x.clamp(
        LEFT_WALL + transform.scale.x / 2.0 + WALL_THICKNESS,
        RIGHT_WALL - transform.scale.x / 2.0 - WALL_THICKNESS
    );

    // update cooldown timer
    if let Some(cooldown_timer) = &mut shooting_cooldown {
        if cooldown_timer.finished() {
            commands.entity(ship).remove::<ShootingCooldown>();
        } else {
            cooldown_timer.tick(Duration::from_secs_f32(TIME_STEP));
        }
    }

    if shoot && shooting_cooldown.is_none() {
        let bullet_x;
        
        // randomly shoot from left or right extent
        if random::<f32>() < 0.5 {
            bullet_x = transform.translation.x + SHIP_SIZE.x / 2.;
        } else {
            bullet_x = transform.translation.x - SHIP_SIZE.x / 2.;
        } 

        let bullet_y = transform.translation.y + transform.scale.y / 2. + SHIP_BULLET_INITIAL_GAP;

        commands
            .entity(ship)
            .insert(ShootingCooldown(Timer::from_seconds(SHOOTING_COOLDOWN_IN_SECONDS, false)));

        commands
            .spawn()
            .insert_bundle(BulletBundle::from_ship(
                Vec2::new(bullet_x, bullet_y), 
                sprites.get("FERRIS_BULLET".to_string()).unwrap().clone()
            ));
    }
}
