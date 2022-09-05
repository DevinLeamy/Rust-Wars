use crate::{shared::*, GameState};
use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;
use rand::random;
use std::time::Duration;

const SHIP_SIZE: Vec2 = Vec2::new(120., 80.);
const GAP_BETWEEN_SHIP_AND_FLOOR: f32 = 5.0;
const SHIP_SPEED: f32 = 450.;
const SHOOTING_COOLDOWN_IN_SECONDS: f32 = 0.8;
pub const SHIP_BULLET_SIZE: Vec2 = Vec2::new(33.0, 33.0);
pub const INITIAL_HEALTH_POINTS: u32 = 5;
pub const SHIP_WALK_FRAME_DURATION_IN_MILLIS: u64 = 200;

const HEART_SIZE: Vec2 = Vec2::new(30., 30.);
const HEART_CORNER_OFFSET: Vec2 = Vec2::new(25., 25.);
const HEART_PADDING_RIGHT: f32 = 10.0;

#[derive(Component)]
pub struct Ship;

#[derive(Component, PartialEq, Eq)]
pub enum FerrisState {
    WALKING,
    IDLE,
    DEAD,
}

#[derive(Component)]
struct HealthDisplayHeart(u32);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let mut fixedupdate = SystemStage::parallel();
        fixedupdate.add_system(update_ship.run_in_state(GameState::Playing));
        fixedupdate.add_system(check_for_ship_collisions.run_in_state(GameState::Playing));
        fixedupdate.add_system(update_health_display.run_in_state(GameState::Playing));
        fixedupdate.add_system(update_ferris_display.run_in_state(GameState::Playing));

        app.add_stage_before(
            CoreStage::Update,
            "Player_FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate),
        )
        .add_startup_system(load_assets_and_animations)
        .add_enter_system(GameState::Playing, spawn_ship_health_display)
        .add_enter_system(GameState::LoadWaveState, spawn_player);
    }
}

fn check_for_ship_collisions(
    mut commands: Commands,
    mut ship_query: Query<(&Transform, &mut Health, &Collider, &mut FerrisState), With<Ship>>,
    bullet_query: Query<(Entity, &Transform, &Bullet, &Collider)>,
) {
    let (ship_transform, mut health, ship_collider, mut ferris_state) = ship_query.single_mut();

    for (bullet_entity, bullet_transform, bullet, bullet_collider) in bullet_query.iter() {
        if bullet == &Bullet::Ship {
            // ignore bullets from the ship
            continue;
        }
        if collide(
            ship_transform.translation,
            ship_collider.size,
            bullet_transform.translation,
            bullet_collider.size,
        ).is_some() {
            commands.entity(bullet_entity).despawn();
            health.0 -= 1;

            if health.0 == 0 {
                *ferris_state = FerrisState::DEAD
            }
            break;
        }
    }
}

fn load_assets_and_animations(
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Sprites>,
    mut animations: ResMut<Animations>,
) {
    sprites.add("HEART", asset_server.load("images/heart.png"));
    sprites.add("ALARMED_FERRIS", asset_server.load("images/alarmed_ferris.png"));
    sprites.add("HAPPY_FERRIS", asset_server.load("images/ferris.png"));
    sprites.add("FERRIS_BULLET_FLASH", asset_server.load("images/ferris_bullet_flash.png"));
    sprites.add("FERRIS_WALK_1", asset_server.load("images/ferris_walk/ferris_walk_1.png"));
    sprites.add("FERRIS_WALK_2", asset_server.load("images/ferris_walk/ferris_walk_2.png"));

    let ferris_walk_animation = Animation::from_images(
        vec!["FERRIS_WALK_1".to_string(), "FERRIS_WALK_2".to_string()],
        SHIP_WALK_FRAME_DURATION_IN_MILLIS
    );
    animations.add("FERRIS_WALK", ferris_walk_animation);
}

fn spawn_ship_health_display(mut commands: Commands, sprites: ResMut<Sprites>) {
    let first_heart_offset = Vec2::new(
        LEFT_WALL + HEART_CORNER_OFFSET.x,
        TOP_WALL - HEART_CORNER_OFFSET.y,
    );
    let heart_horizontal_gap = HEART_PADDING_RIGHT + HEART_SIZE.x;

    for i in 0..INITIAL_HEALTH_POINTS {
        let heart_x = first_heart_offset.x + heart_horizontal_gap * i as f32;
        let heart_y = first_heart_offset.y - HEART_SIZE.y;

        commands
            .spawn()
            .insert(HealthDisplayHeart(i + 1))
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(heart_x, heart_y, 0.0),
                    scale: Vec3::splat(1.0),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(HEART_SIZE),
                    ..default()
                },
                texture: sprites.get("HEART"),
                ..default()
            })
            .insert(Name::new("Health Display Heart"));
    }
}

fn update_health_display(
    mut commands: Commands,
    ship_query: Query<&Health, With<Ship>>,
    hearts_query: Query<(Entity, &HealthDisplayHeart)>,
) {
    let health = ship_query.single();

    for (heart_entity, display_heart) in hearts_query.iter() {
        if display_heart.0 > health.0 {
            commands.entity(heart_entity).despawn();
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    sprites: Res<Sprites>,
    animations: Res<Animations>,
    ship_query: Query<Entity, With<Ship>>,
) {
    if ship_query.get_single().is_ok() {
        // ship has already been spawned
        return;
    }

    // ship
    let ship_y = BOTTOM_WALL + GAP_BETWEEN_SHIP_AND_FLOOR + SHIP_SIZE.y / 2.;

    commands
        .spawn()
        .insert(Ship)
        .insert(Health(INITIAL_HEALTH_POINTS))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, ship_y, 0.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(SHIP_SIZE),
                ..default()
            },
            texture: sprites.get("HAPPY_FERRIS"),
            ..default()
        })
        .insert(ShootingCooldown::new(DurationType::Fixed(Fixed(SHOOTING_COOLDOWN_IN_SECONDS))))
        .insert(animations.get("FERRIS_WALK").animation)
        .insert(AnimationState::default())
        .insert(FerrisState::IDLE)
        .insert(Collider { size: SHIP_SIZE });
}

fn update_ship(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<
        (
            &mut Transform,
            &mut FerrisState,
            &mut ShootingCooldown,
            &Collider,
        ),
        With<Ship>,
    >,
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    let (mut transform, mut state, mut shooting_cooldown, collider) = query.single_mut();

    if *state == FerrisState::DEAD {
        return;
    }

    let mut direction = 0.;

    let move_left = keyboard_input.pressed(KeyCode::A);
    let move_right = keyboard_input.pressed(KeyCode::D);
    let shoot = keyboard_input.pressed(KeyCode::Space);

    if move_left {
        direction = -1.;
        *state = FerrisState::WALKING;
    } else if move_right {
        direction = 1.;
        *state = FerrisState::WALKING;
    } else {
        *state = FerrisState::IDLE;
    }

    transform.translation.x += direction * SHIP_SPEED * TIME_STEP;

    transform.translation.x = transform.translation.x.clamp(
        LEFT_WALL + collider.size.x / 2.0 + WALL_THICKNESS,
        RIGHT_WALL - collider.size.x / 2.0 - WALL_THICKNESS,
    );

    if shoot && shooting_cooldown.finished() {
        shooting_cooldown.reset();

        let offset = if random::<f32>() < 0.5 { 1.0 } else { -1.0 };
        let bullet_x = transform.translation.x + offset * SHIP_SIZE.x / 2.;
        let bullet_y = transform.translation.y + transform.scale.y / 2. + SHIP_BULLET_INITIAL_GAP;

        commands.spawn().insert_bundle(BulletBundle::from_ship(
            Vec2::new(bullet_x, bullet_y),
            sprites.get("FERRIS_BULLET"),
        ));
    }
}

fn update_ferris_display(
    mut query: Query<
        (
            &mut AnimationState,
            &BAnimation,
            &mut Handle<Image>,
            &FerrisState,
        ),
        With<Ship>,
    >,
    sprites: Res<Sprites>,
    animations: Res<Animations>,
) {
    let ferris_walk_animation = animations.get("FERRIS_WALK");
    let images = match &ferris_walk_animation.image_data {
        ImageData::Images(images) => images,
        _ => panic!("Image data not found"),
    };

    let (mut animation_state, ferris_animation, mut texture, ferris_state) = query.single_mut();

    match ferris_state {
        FerrisState::IDLE => {
            *texture = sprites.get("HAPPY_FERRIS");
        }
        FerrisState::WALKING => {
            animation_state.update(ferris_animation, Duration::from_secs_f32(TIME_STEP));
            *texture = sprites.get(images[animation_state.frame_index() as usize].as_str());
        }
        FerrisState::DEAD => {
            *texture = sprites.get("ALARMED_FERRIS");
        }
    }
}
