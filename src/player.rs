use crate::{shared::*, GameState};
use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;
use std::time::Duration;

const SHIP_SIZE: Vec2 = Vec2::new(120., 80.);
const SHIP_COLLISION_SIZE: Vec2 = Vec2::new(110., 70.);
const GAP_BETWEEN_SHIP_AND_FLOOR: f32 = 5.0;
const SHIP_SPEED: f32 = 450.;
const SHOOTING_COOLDOWN_IN_SECONDS: f32 = 1.2;
pub const SHIP_BULLET_SIZE: Vec2 = Vec2::new(33.0, 70.0);
pub const SHIP_BULLET_FLASH_SIZE: Vec2 = Vec2::new(33.0, 70.0);
pub const INITIAL_HEALTH_POINTS: u32 = 8;
pub const SHIP_WALK_FRAME_DURATION_IN_MILLIS: u64 = 200;
pub const HIT_MARKER_SIZE: Vec2 =  Vec2::new(25.0, 25.0); 
pub const HIT_MARKER_DURATION: f32 = 0.75;

const HEART_SIZE: Vec2 = Vec2::new(30., 30.);
const HEART_CORNER_OFFSET: Vec2 = Vec2::new(25., 25.);
const HEART_PADDING_RIGHT: f32 = 10.0;

#[derive(Component, PartialEq)]
enum Torch {
    Left,
    Right
}

#[derive(Component)]
pub struct Ship;

#[derive(Component, PartialEq, Eq)]
pub enum FerrisState {
    WALKING,
    IDLE,
    DEAD,
}

#[derive(Component)]
pub struct HealthDisplayHeart(u32);

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
    mut ship_query: Query<(Entity, &Transform, &mut Health, &Collider, &mut FerrisState), With<Ship>>,
    sprites: Res<Sprites>,
    bullet_query: Query<(Entity, &Transform, &Bullet, &Collider)>,
) {
    let (ship_entity, ship_transform, mut health, ship_collider, mut ferris_state) = ship_query.single_mut();

    let min_bullet_hit_height = ship_transform.translation.y;

    for (bullet_entity, bullet_transform, bullet, bullet_collider) in bullet_query.iter() {
        if bullet == &Bullet::Ship {
            // ignore bullets from the ship
            continue;
        }

        let bullet_translation = bullet_transform.translation;

        if bullet_translation.y < min_bullet_hit_height {
            continue;
        }

        if collide(
            ship_transform.translation,
            ship_collider.size,
            bullet_translation,
            bullet_collider.size,
        ).is_some() {
            commands.entity(bullet_entity).despawn();

            let hit_marker = commands
                    .spawn()
                    .insert_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(
                                bullet_translation.x - ship_transform.translation.x,
                                bullet_translation.y - ship_transform.translation.y - 20.0,
                                1.0
                            ),
                            ..default()
                        },
                        texture: sprites.get("HIT_MARKER"),
                        sprite: Sprite {
                            custom_size: Some(HIT_MARKER_SIZE),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(DespawnTimer::from_seconds(HIT_MARKER_DURATION))
                    .id();
            
            commands.entity(ship_entity).add_child(hit_marker);

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
    sprites.add("HIT_MARKER", asset_server.load("images/hit_marker.png"));
    sprites.add("ALARMED_FERRIS", asset_server.load("images/alarmed_ferris.png"));
    sprites.add("HAPPY_FERRIS", asset_server.load("images/ferris.png"));
    sprites.add("FERRIS_BULLET", asset_server.load("images/ferris_bullet.png"));
    sprites.add("FERRIS_BULLET_FLASH", asset_server.load("images/ferris_bullet.png"));
    sprites.add("FERRIS_WALK_1", asset_server.load("images/ferris_walk/ferris_walk_1.png"));
    sprites.add("FERRIS_WALK_2", asset_server.load("images/ferris_walk/ferris_walk_2.png"));

    let ferris_walk_animation = Animation::from_images(
        vec!["FERRIS_WALK_1".to_string(), "FERRIS_WALK_2".to_string()],
        SHIP_WALK_FRAME_DURATION_IN_MILLIS
    );
    animations.add("FERRIS_WALK", ferris_walk_animation);
}

fn get_left_claw_offset() -> Vec2 {
    Vec2::new(-1.0 * SHIP_SIZE.x / 2.0 + 10.0, 15.0)
}

fn get_right_claw_offset() -> Vec2 {
    Vec2::new(1.0 * SHIP_SIZE.x / 2.0 - 10.0, 15.0)
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

    let ferris = commands
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
        .insert(animations.get("FERRIS_WALK").animation)
        .insert(AnimationState::default())
        .insert(FerrisState::IDLE)
        .insert(Collider { size: SHIP_COLLISION_SIZE})
        .id();
    
    let left_torch = commands
        .spawn()
        .insert(ShootingCooldown::new_finished(DurationType::Fixed(Fixed(SHOOTING_COOLDOWN_IN_SECONDS))))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: get_left_claw_offset().extend(1.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(SHIP_BULLET_FLASH_SIZE),
                ..default()
            },
            texture: sprites.get("FERRIS_BULLET_FLASH"),
            ..default()
        })
        .insert(Torch::Left)
        .insert(Visibility { is_visible: true })
        .id(); 

    let right_torche = commands
        .spawn()
        .insert(ShootingCooldown::new_finished(DurationType::Fixed(Fixed(SHOOTING_COOLDOWN_IN_SECONDS))))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: get_right_claw_offset().extend(1.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(SHIP_BULLET_FLASH_SIZE),
                ..default()
            },
            texture: sprites.get("FERRIS_BULLET_FLASH"),
            ..default()
        })
        .insert(Torch::Right)
        .insert(Visibility { is_visible: true })
        .id();  
    
    commands.entity(ferris).add_child(left_torch);
    commands.entity(ferris).add_child(right_torche);
}

fn update_ship(
    keyboard_input: Res<Input<KeyCode>>,
    mut ship_query: Query<
        (
            &mut Transform,
            &Children,
            &mut FerrisState,
            &Collider,
        ),
        With<Ship>,
    >,
    mut torch_query: Query<(&Transform, &mut Visibility, &mut ShootingCooldown, &Torch), Without<Ship>>,
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    let (mut transform, children, mut state, collider) = ship_query.single_mut();

    if *state == FerrisState::DEAD {
        return;
    }

    let mut direction = 0.;

    let move_left = keyboard_input.pressed(KeyCode::A);
    let move_right = keyboard_input.pressed(KeyCode::D);
    let shoot_left = keyboard_input.pressed(KeyCode::J);
    let shoot_right = keyboard_input.pressed(KeyCode::K);

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

    // update torchs (only show if you can fire)
    for child in children {
        if let Ok((torch_transform, mut torch_visibility, mut torch_cooldown, torch)) = torch_query.get_mut(*child) {
            torch_visibility.is_visible = torch_cooldown.finished();

            if torch_cooldown.finished() && (shoot_left && torch == &Torch::Left || shoot_right && torch == &Torch::Right) {
                torch_cooldown.reset();

                let bullet_offset = torch_transform.translation.truncate(); 

                let bullet_x = transform.translation.x + bullet_offset.x;
                let bullet_y = transform.translation.y + bullet_offset.y;

                commands.spawn().insert_bundle(BulletBundle::from_ship(
                    Vec2::new(bullet_x, bullet_y),
                    sprites.get("FERRIS_BULLET"),
                )); 
            } 
        }
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
