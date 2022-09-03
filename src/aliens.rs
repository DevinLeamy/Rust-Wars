use std::{time::Duration};
use benimator::FrameRate;
use rand::*;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;

use crate::{shared::*, Scoreboard, Explosion, GameState};


// alien
const ALIEN_ODD_ROW_OFFSET: f32 = 30.0;
const ALIEN_WALL_GAP: f32 = 20.;
pub const ALIEN_SIZE: Vec2 = Vec2::new(69.0, 46.);
const ALIEN_SPEED: f32 = 75.;
const ALIEN_ALIEN_GAP: Vec2 = Vec2::new(30., 50.);
pub const ALIEN_BULLET_SPEED: f32 = 300.0;
const INITIAL_ALIEN_DIRECTION: f32 = 1.; // right
const DESTROY_ALIEN_SCORE: u32 = 5;
const MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 15.;
pub const ALIEN_WALK_FRAME_DURATION_IN_MILLIS: u64 = 200;
pub const BULLET_FLASH_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const BULLET_FLASH_DURATION_IN_SECONDS: f32 = 0.1;

#[derive(Component)]
pub struct Alien;

pub struct AliensPlugin;

impl Plugin for AliensPlugin {
    fn build(&self, app: &mut App) {
        let mut fixedupdate = SystemStage::parallel();
        fixedupdate.add_system(update_aliens.run_in_state(GameState::Playing));
        fixedupdate.add_system(update_alien_animations.run_in_state(GameState::Playing));
        fixedupdate.add_system(check_for_alien_collisions.run_in_state(GameState::Playing));

        app
            .add_stage_before(
                CoreStage::Update,
                "Alien_FixedUpdate",
                FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate)
            )
            .add_startup_system(load_assets)
            .add_enter_system(GameState::Playing, spawn_aliens);
    }
}

fn load_assets(asset_server: Res<AssetServer>, mut sprites: ResMut<Sprites>) {
    sprites.add("ALIEN_BULLET".to_string(), asset_server.load("images/alien_bullet/bullet.png"));
    sprites.add("ALIEN_BULLET_FLASH".to_string(), asset_server.load("images/alien_bullet/bullet_flash.png"));
    sprites.add("ALIEN_WALK_1".to_string(), asset_server.load("images/alien_ferris/walk_1.png"));
    sprites.add("ALIEN_WALK_2".to_string(), asset_server.load("images/alien_ferris/walk_2.png"));
}

fn spawn_aliens(
    mut commands: Commands,
    mut animations: ResMut<Animations>,
    sprites: Res<Sprites>
) {
    let alien_walk_1 = "ALIEN_WALK_1".to_string();
    let alien_walk_2 = "ALIEN_WALK_2".to_string(); 

    let alien_animation = Animation {
        animation: BAnimation(benimator::Animation::from_indices(
            0..2,
            FrameRate::from_frame_duration(Duration::from_millis(ALIEN_WALK_FRAME_DURATION_IN_MILLIS))
        )),
        image_data: ImageData::Images(vec![alien_walk_1.clone(), alien_walk_2.clone()])
    };

    animations.add("ALIEN_WALK".to_string(), alien_animation);

    let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP + ALIEN_SIZE.x / 2.;
    let first_alien_y = TOP_WALL - ALIEN_WALL_GAP  - ALIEN_SIZE.y / 2. - 80.;

    let total_alien_width = ALIEN_SIZE.x + ALIEN_ALIEN_GAP.x;
    let total_alien_height = ALIEN_SIZE.y + ALIEN_ALIEN_GAP.y;

    // spawn aliens
    for row in 0..5 {
        for col in 0..(5 + row % 2) {
            let alien_y = first_alien_y - row as f32 * total_alien_height; 

            let alien_x;

            if row % 2 == 0 { 
                alien_x = first_alien_x + col as f32 * total_alien_width; 
            } else {
                alien_x = first_alien_x + col as f32 * total_alien_width - ALIEN_ODD_ROW_OFFSET; 
            }

            commands
                .spawn()
                .insert(Alien)
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(alien_x, alien_y, 0.0),
                        scale: Vec3::splat(1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(ALIEN_SIZE),
                        ..default()
                    }, 
                    texture: sprites.get(alien_walk_1.clone()),
                    ..default()
                })
                .insert(Name::new("Alien"))
                .insert(animations.get("ALIEN_WALK".to_string()).animation)
                .insert(ShootingCooldown(Timer::from_seconds(random::<f32>() * MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS, false)))
                .insert(Velocity(Vec2::new(ALIEN_SPEED * INITIAL_ALIEN_DIRECTION, 0.0)))
                .insert(AnimationState::default())
                .insert(Collider { size: ALIEN_SIZE });
        }
    }
}

fn update_aliens(
    mut alien_query: Query<(Entity, &mut Transform, &mut Velocity, &mut ShootingCooldown), With<Alien>>,
    mut commands: Commands,
    sprites: Res<Sprites>
) {
    let alien_forward_shift = ALIEN_ALIEN_GAP.y / 2. + ALIEN_SIZE.y / 2.;

    for (alien_entity, mut transform, mut velocity, mut shooting_cooldown) in &mut alien_query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;

        let left_most_side = transform.translation.x - transform.scale.x / 2.;
        let right_most_side = transform.translation.x + transform.scale.x / 2.;

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
