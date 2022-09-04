use std::{time::Duration};
use benimator::FrameRate;
use bevy_tweening::{*, lens::TransformPositionLens}; 
use rand::*;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;

use crate::{shared::*, Scoreboard, Explosion, GameState, LOAD_WAVE_DURATION_IN_SECONDS, Global};

// Alien::Aris alien
const ALIEN_ODD_ROW_OFFSET: f32 = 30.0;
const ALIEN_WALL_GAP: Vec2 = Vec2::new(20.0, 20.0);
pub const ALIEN_SIZE: Vec2 = Vec2::new(60.0, 40.);
const ALIEN_SPEED: f32 = 75.;
const ALIEN_ALIEN_GAP: Vec2 = Vec2::new(20., 40.);
pub const ALIEN_BULLET_SPEED: f32 = 300.0;
const INITIAL_ALIEN_DIRECTION: f32 = 1.; // right
const DESTROY_ALIEN_SCORE: u32 = 5;
const MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 10.;
pub const ALIEN_WALK_FRAME_DURATION_IN_MILLIS: u64 = 200;
pub const BULLET_FLASH_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const BULLET_FLASH_DURATION_IN_SECONDS: f32 = 0.1;

#[derive(Component)]
pub enum Alien {
    Aris,
    Raket
}

pub struct AliensPlugin;

impl Plugin for AliensPlugin {
    fn build(&self, app: &mut App) {
        let mut fixedupdate = SystemStage::parallel();
        fixedupdate.add_system(update_aliens.run_in_state(GameState::Playing));
        fixedupdate.add_system(update_alien_animations.run_in_state(GameState::Playing));
        fixedupdate.add_system(update_alien_animations.run_in_state(GameState::LoadWaveState));
        fixedupdate.add_system(check_for_alien_collisions.run_in_state(GameState::Playing));

        app
            .add_stage_before(
                CoreStage::Update,
                "Alien Fixed Timestep",
                FixedTimestepStage::from_stage(
                    Duration::from_secs_f32(TIME_STEP), 
                    fixedupdate
                )
            )
            .add_startup_system(load_assets)
            .add_enter_system(GameState::LoadWaveState, spawn_aliens);
    }
}

fn load_assets(asset_server: Res<AssetServer>, mut sprites: ResMut<Sprites>, mut animations: ResMut<Animations>) {
    sprites.add("ALIEN_BULLET".to_string(), asset_server.load("images/alien_bullet/bullet.png"));
    sprites.add("ALIEN_BULLET_FLASH".to_string(), asset_server.load("images/alien_bullet/bullet_flash.png"));
    sprites.add("ALIEN_WALK_1".to_string(), asset_server.load("images/alien_ferris/walk_1.png"));
    sprites.add("ALIEN_WALK_2".to_string(), asset_server.load("images/alien_ferris/walk_2.png"));

    let alien_animation = Animation {
        animation: BAnimation(benimator::Animation::from_indices(
            0..2,
            FrameRate::from_frame_duration(Duration::from_millis(ALIEN_WALK_FRAME_DURATION_IN_MILLIS))
        )),
        image_data: ImageData::Images(vec!["ALIEN_WALK_1".to_string(), "ALIEN_WALK_2".to_string()])
    };

    animations.add("ALIEN_WALK".to_string(), alien_animation);
}

#[derive(Bundle)]
struct AlienBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
    alien: Alien,
    shooting_cooldown: ShootingCooldown,
    velocity: Velocity,
    name: Name
}

impl AlienBundle {
    fn new_aris(size: Vec2, translation: Vec2, max_shooting_cooldown_time: f32, velocity: Vec2, texture: Handle<Image>) -> AlienBundle {
        AlienBundle {
            alien: Alien::Aris,
            sprite_bundle: SpriteBundle {
                transform: Transform { translation: translation.extend(0.0), ..default() },
                sprite: Sprite { custom_size: Some(size), ..default() },
                texture: texture,
                ..default()
            },
            collider: Collider { size: size },
            shooting_cooldown: ShootingCooldown(Timer::from_seconds(random::<f32>() * max_shooting_cooldown_time, false)),
            velocity: Velocity(velocity),
            name: Name::new("Alien")
        }
    } 
}

fn wave_zero(mut commands: Commands, animations: Res<Animations>, sprites: Res<Sprites>) {
    let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP.x + ALIEN_SIZE.x / 2.;
    let first_alien_y = TOP_WALL - ALIEN_WALL_GAP.y - ALIEN_SIZE.y / 2. - 80.;

    let total_alien_width = ALIEN_SIZE.x + ALIEN_ALIEN_GAP.x;
    let total_alien_height = ALIEN_SIZE.y + ALIEN_ALIEN_GAP.y;

    // spawn aliens
    for row in 0..5 {
        if row == 2 {
            continue;
        }
        for col in 0..(9 + row % 2) {
            if (row == 1 || row == 3) && col % 3 == 0 {
                continue;
            }
            let alien_x = first_alien_x + col as f32 * total_alien_width - ALIEN_ODD_ROW_OFFSET * ((row % 2) as f32); 
            let alien_y = first_alien_y - row as f32 * total_alien_height; 

            let starting_x = LEFT_WALL + (random::<f32>() * WINDOW_WIDTH);
            let starting_y = BOTTOM_WALL + WINDOW_HEIGHT / 2.0 + (random::<f32>() * WINDOW_HEIGHT);

            let position_tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs_f32(LOAD_WAVE_DURATION_IN_SECONDS * f32::min(1.0, random::<f32>() + 0.25)),
                TransformPositionLens {
                    start: Vec3::new(starting_x, starting_y, 0.0),
                    end: Vec3::new(alien_x, alien_y, 0.0),
                },
            );

            commands
                .spawn()
                .insert_bundle(AlienBundle::new_aris(
                    ALIEN_SIZE,
                    Vec2::new(starting_x, starting_y),
                    MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS,
                    Vec2::new(ALIEN_SPEED * INITIAL_ALIEN_DIRECTION, 0.0),
                    sprites.get("ALIEN_WALK_1".to_string()) 
                ))
                .insert(Animator::new(position_tween))
                .insert(animations.get("ALIEN_WALK".to_string()).animation)
                .insert(AnimationState::default());
        }
    }
}

fn wave_one(mut commands: Commands, animations: Res<Animations>, sprites: Res<Sprites>) {
    println!("Wave one");
}

fn spawn_aliens(commands: Commands, animations: Res<Animations>, sprites: Res<Sprites>, global: Res<Global>) {
    match global.current_wave() {
        0 => wave_zero(commands, animations, sprites), 
        1 => wave_one(commands, animations, sprites),
        _ => panic!("Wave not implemented") 
    }
}

fn update_aliens(
    mut alien_query: Query<(Entity, &mut Transform, &mut Velocity, &mut ShootingCooldown, &Collider), With<Alien>>,
    mut commands: Commands,
    sprites: Res<Sprites>
) {
    let alien_forward_shift = ALIEN_ALIEN_GAP.y / 2. + ALIEN_SIZE.y / 2.;

    for (alien_entity, mut transform, mut velocity, mut shooting_cooldown, collider) in &mut alien_query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;

        let left_most_side = transform.translation.x - collider.size.x / 2.;
        let right_most_side = transform.translation.x + collider.size.x / 2.;

        // Wall checks are intentionally done this way.
        // Gives the impression of shifting off and then back onto the screen.
        if right_most_side < LEFT_WALL || left_most_side > RIGHT_WALL {
            velocity.x *= -1.;
            transform.translation.y -= alien_forward_shift;
        }

        // update cooldown timer
        if shooting_cooldown.finished() {
            let offset = if random::<f32>() < 0.5 { 1.0 } else { -1.0 };
            let bullet_x = transform.translation.x + offset * ALIEN_SIZE.x / 2.;
            let bullet_y = transform.translation.y - ALIEN_SIZE.y / 4.; 

            commands
                .spawn()
                .insert_bundle(BulletBundle::from_alien(
                    Vec2::new(bullet_x, bullet_y),
                    sprites.get("ALIEN_BULLET".to_string())
                ));
            
            let bullet_flash = commands
                .spawn()
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec2::new(offset * ALIEN_SIZE.x / 2., 0.).extend(1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(BULLET_FLASH_SIZE),
                        ..default()
                    },
                    texture: sprites.get("ALIEN_BULLET_FLASH".to_string()),
                    ..default()
                })
                .insert(DespawnTimer(Timer::from_seconds(BULLET_FLASH_DURATION_IN_SECONDS, false)))
                .id();
            
            commands.entity(alien_entity).add_child(bullet_flash);

            shooting_cooldown.reset();
            shooting_cooldown.set_duration(Duration::from_secs_f32(random::<f32>() * MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS))
        } else {
            shooting_cooldown.tick(Duration::from_secs_f32(TIME_STEP));
        }
    } 
}

fn check_for_alien_collisions(
    mut scoreboard: ResMut<Scoreboard>,
    alien_query: Query<(Entity, &Transform, &Collider), With<Alien>>, 
    bullet_query: Query<(Entity, &Bullet, &Transform, &Collider)>,
    animations: Res<Animations>,
    mut commands: Commands,
) {
    for (alien_entity, transform, alien_collider) in &alien_query {
        for (bullet_entity, bullet, bullet_transform, bullet_collider) in &bullet_query {
            if bullet == &Bullet::Alien {
                // ignore bullets from other aliens
                continue;
            }
            if let Some(_collision) = collide(
                transform.translation,
                alien_collider.size,
                bullet_transform.translation,
                bullet_collider.size
            ) {
                commands.entity(bullet_entity).despawn_recursive();
                commands.entity(alien_entity).despawn_recursive();

                let explosion = animations.get("EXPLOSION".to_string());
                let texture_atlas = match &explosion.image_data { 
                    ImageData::TextureAtlas(texture_atlas) => texture_atlas, 
                    _                                      => panic!("Explosion is stored as a texture atlas!")
                };

                commands
                    .spawn()
                    .insert_bundle(SpriteSheetBundle {
                        texture_atlas: texture_atlas.clone(),
                        transform: Transform {
                            translation: bullet_transform.translation,
                            scale: Vec3::splat(EXPLOSION_SIZE),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(explosion.animation.clone())
                    .insert(AnimationState::default())
                    .insert(Explosion);
                    

                scoreboard.score += DESTROY_ALIEN_SCORE;
                break;
            }
        }
    }
}

fn update_alien_animations (
    mut query: Query<(&mut AnimationState, &BAnimation, &mut Handle<Image>), With<Alien>>,
    sprites: Res<Sprites>,
    animations: Res<Animations>, 
) {
    let alien_walk_animation = animations.get("ALIEN_WALK".to_string());
    let images = match &alien_walk_animation.image_data {
        ImageData::Images(images) => images,
        _                         => panic!("Image data not found")
    };

    for (mut animation_state, alien_animation, mut texture) in query.iter_mut() {
        animation_state.update(alien_animation, Duration::from_secs_f32(TIME_STEP));
        *texture = sprites.get(images[animation_state.frame_index() as usize].clone())
    }
}
