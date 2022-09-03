use benimator::FrameRate;
use bevy::{prelude::*, sprite::collide_aabb::collide};
use rand::random;
use crate::{shared::*, aliens::ALIEN_SIZE, GameState};
use std::time::Duration;
use iyes_loopless::prelude::*;

pub const SHIP_BULLET_IMAGE_SIZE: Vec2 = Vec2::new(512.0, 512.0);
const SHIP_IMAGE_SIZE: Vec2 = Vec2::new(1200.0, 800.0);
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
    DEAD
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

        app
            .add_stage_before(
                CoreStage::Update,
                "Player_FixedUpdate",
                FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate)
            )
            .add_enter_system(GameState::Playing, spawn_ship_health_display)
            .add_enter_system(GameState::Playing, spawn_player);
    }
}

fn check_for_ship_collisions(
    mut commands: Commands,
    mut ship_query: Query<(&Transform, &mut Health, &mut FerrisState), With<Ship>>,
    bullet_query: Query<(Entity, &Transform, &Bullet)>,
) {
    let (ship_transform, mut health, mut ferris_state) = ship_query.single_mut();

    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        if bullet == &Bullet::Ship {
            // ignore bullets from the ship 
            continue;
        }
        if let Some(_collision) = collide(
            ship_transform.translation,
            ship_transform.scale.truncate(),
            bullet_transform.translation,
            bullet_transform.scale.truncate(),
        ) {
            commands.entity(bullet_entity).despawn();
            health.0 -= 1;

            if health.0 == 0 {
                *ferris_state = FerrisState::DEAD
            }
            break;
        } 
    }
}

fn spawn_ship_health_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Sprites>
) {
    sprites.add("HEART".to_string(), asset_server.load("images/heart.png"));

    let first_heart_offset = Vec2::new(LEFT_WALL + HEART_CORNER_OFFSET.x, TOP_WALL - HEART_CORNER_OFFSET.y); 
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
                texture: sprites.get("HEART".to_string()),
                ..default()
            })
            .insert(Name::new("Health Display Heart"));
    }
}

fn update_health_display(
    mut commands: Commands,
    ship_query: Query<&Health, With<Ship>>,
    hearts_query: Query<(Entity, &HealthDisplayHeart)>
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
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Sprites>,
    mut animations: ResMut<Animations>
) {
    sprites.add("ALARMED_FERRIS".to_string(), asset_server.load("images/alarmed_ferris.png"));
    sprites.add("HAPPY_FERRIS".to_string(), asset_server.load("images/ferris.png"));

    let ferris_walk_animation = Animation {
        animation: BAnimation(benimator::Animation::from_indices(
            0..2,
            FrameRate::from_frame_duration(Duration::from_millis(SHIP_WALK_FRAME_DURATION_IN_MILLIS))
        )),
        image_data: ImageData::Images(vec!["FERRIS_WALK_1".to_string(), "FERRIS_WALK_2".to_string()])
    };

    sprites.add("FERRIS_WALK_1".to_string(), asset_server.load("images/ferris_walk/ferris_walk_1.png"));
    sprites.add("FERRIS_WALK_2".to_string(), asset_server.load("images/ferris_walk/ferris_walk_2.png"));

    animations.add("FERRIS_WALK".to_string(), ferris_walk_animation);

     // ship 
     let ship_y = BOTTOM_WALL + GAP_BETWEEN_SHIP_AND_FLOOR + SHIP_SIZE.y / 2.;

     commands
         .spawn()
         .insert(Ship)
         .insert(Health(INITIAL_HEALTH_POINTS))
         .insert_bundle(SpriteBundle {
             transform: Transform {
                 translation: Vec3::new(0.0, ship_y, 0.0),
                 scale: SHIP_SIZE.extend(1.0),
                 ..default()
             },
             sprite: generate_texture_sprite(ALIEN_SIZE, SHIP_IMAGE_SIZE), 
             texture: sprites.get("HAPPY_FERRIS".to_string()),
             ..default()
         })
         .insert(animations.get("FERRIS_WALK".to_string()).animation)
         .insert(AnimationState::default())
         .insert(FerrisState::IDLE)
         .insert(Collider);
}

fn update_ship(
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<(Entity, &mut Transform, &mut FerrisState, Option<&mut ShootingCooldown>), With<Ship>>, 
    sprites: Res<Sprites>,
    mut commands: Commands
) {
    let (ship, mut transform, mut state, mut shooting_cooldown) = query.single_mut(); 

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
                sprites.get("FERRIS_BULLET".to_string())
            ));
    }
}


fn update_ferris_display (
    mut query: Query<(&mut AnimationState, &BAnimation, &mut Handle<Image>, &FerrisState), With<Ship>>,
    sprites: Res<Sprites>,
    animations: Res<Animations>, 
) {
    let ferris_walk_animation = animations.get("FERRIS_WALK".to_string());
    let images = match &ferris_walk_animation.image_data {
        ImageData::Images(images) => images,
        _                         => panic!("Image data not found")
    };

    let (mut animation_state, ferris_animation, mut texture, ferris_state) = query.single_mut();

    match ferris_state {
        FerrisState::IDLE => {
            *texture = sprites.get("HAPPY_FERRIS".to_string());
        }
        FerrisState::WALKING => {
            animation_state.update(ferris_animation, Duration::from_secs_f32(TIME_STEP));
            *texture = sprites.get(images[animation_state.frame_index() as usize].clone());
        },
        FerrisState::DEAD => {
            *texture = sprites.get("ALARMED_FERRIS".to_string());
        }
    }
}
