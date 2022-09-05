use benimator::FrameRate;
use bevy_tweening::{lens::TransformPositionLens, *};
use rand::*;
use std::time::Duration;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;

use crate::{shared::*, Explosion, GameState, Global, Scoreboard, LOAD_WAVE_DURATION_IN_SECONDS};

// Alien::Aris alien
const ALIEN_ODD_ROW_OFFSET: f32 = 30.0;
const ALIEN_WALL_GAP: Vec2 = Vec2::new(20.0, 20.0);
const ALIEN_ALIEN_GAP: Vec2 = Vec2::new(20., 40.);

const DESTROY_ALIEN_SCORE: u32 = 5;

pub const ALIEN_WALK_FRAME_DURATION_IN_MILLIS: u64 = 200;
pub const BULLET_FLASH_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const BULLET_FLASH_DURATION_IN_SECONDS: f32 = 0.1;

#[derive(Component)]
pub struct Alien;

#[derive(Component)]
pub struct Aris;

impl Aris {
    pub const SIZE: Vec2 = Vec2::new(60.0, 40.);
    pub const SPEED: f32 = 75.;
    pub const BULLET_SPEED: f32 = 300.0;
    pub const MAX_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 10.;
    pub const INITIAL_DIRECTION: f32 = 1.; // right
}

#[derive(Component)]
pub struct Rylo;

impl Rylo {
    const SIZE: Vec2 = Vec2::new(40.0, 40.);
    pub const BULLET_SPEED: f32 = 300.0;
    const DESTROY_ALIEN_SCORE: u32 = 10;
    const MAX_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 15.0;
    const POSITION_TWEEN_COMPLETE: u64 = 1;
    const POSITION_TWEEN_DELAY: f32 = 10.0;
    const POSITION_TWEEN_DURATION: f32 = 10.0;
}

impl Rylo {
    fn update(
        mut query: Query<(Entity, &mut Transform, &mut ShootingCooldown), With<Rylo>>,
        mut commands: Commands,
        sprites: Res<Sprites>
    ) {
        for (alien_entity, transform, mut cooldown) in query.iter_mut() {
            cooldown.tick(TIME_STEP);
            // update cooldown timer
            if cooldown.finished() {
                let offset = if random::<f32>() < 0.5 { 1.0 } else { -1.0 };
                let bullet_x = transform.translation.x + offset * Aris::SIZE.x / 2.;
                let bullet_y = transform.translation.y - Aris::SIZE.y / 4.;

                commands.spawn().insert_bundle(BulletBundle::from_rylo(
                    Vec2::new(bullet_x, bullet_y),
                    sprites.get("RYLO_BULLET"),
                ));

                let bullet_flash = commands
                    .spawn()
                    .insert_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec2::new(offset * Aris::SIZE.x / 2., 0.).extend(1.0),
                            ..default()
                        },
                        sprite: Sprite {
                            custom_size: Some(BULLET_FLASH_SIZE),
                            ..default()
                        },
                        texture: sprites.get("RYLO_BULLET_FLASH"),
                        ..default()
                    })
                    .insert(DespawnTimer::from_seconds(BULLET_FLASH_DURATION_IN_SECONDS))
                    .id();

                commands.entity(alien_entity).add_child(bullet_flash);

                cooldown.reset();
            } 
 
        }
   }

    fn update_position_tweens(mut commands: Commands, query: Query<&Transform, With<Rylo>>, mut event_reader: EventReader<TweenCompleted>) {
        for event in event_reader.iter() {
            let entity = event.entity;
            let tween_id = event.user_data;

            if tween_id != Rylo::POSITION_TWEEN_COMPLETE {
                continue;
            }

            let alien_transform = query.get(entity).expect("rylo not found");

            let ending_x = LEFT_WALL + (random::<f32>() * WINDOW_WIDTH);
            let ending_y = BOTTOM_WALL + WINDOW_HEIGHT / 2.0 + (random::<f32>() * WINDOW_HEIGHT / 2.0);

            let position_tween = Rylo::get_position_tween(
                alien_transform.translation.truncate(), 
                Vec2::new(ending_x, ending_y), 
                random::<f32>() * Rylo::POSITION_TWEEN_DURATION); 

            // let deplayed_tween = position_tween.then(
            //     Delay::new(Duration::from_secs_f32(random::<f32>() * Rylo::POSITION_TWEEN_DELAY))
            // );

            commands.entity(entity).insert(Animator::new(position_tween));
        }         
    }

    fn get_position_tween(start: Vec2, end: Vec2, duration: f32) -> Tween<Transform> {
        if random::<f32>() > 0.4 {
            Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs_f32(f32::max(duration, duration * f32::min(1.0, random::<f32>() + 0.25))),
                TransformPositionLens {
                    start: start.extend(0.0), 
                    end: end.extend(0.0) 
                },
            ).with_completed_event(Rylo::POSITION_TWEEN_COMPLETE)
        } else {
            Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs_f32(f32::max(duration, duration * f32::min(1.0, random::<f32>() + 0.25))),
                TransformPositionLens {
                    start: start.extend(0.0), 
                    end: start.extend(0.0) 
                },
            ).with_completed_event(Rylo::POSITION_TWEEN_COMPLETE) 
        }
   }
}

pub struct AliensPlugin;

impl Plugin for AliensPlugin {
    fn build(&self, app: &mut App) {
        let mut fixedupdate = SystemStage::parallel();
        fixedupdate.add_system_set(
            ConditionSet::new()
                .label("Alien Updates")
                .run_in_state(GameState::Playing)
                .with_system(Rylo::update)
                .with_system(update_aris_aliens)
                .into(),
        );
        fixedupdate.add_system(update_alien_animations.run_in_state(GameState::Playing));
        fixedupdate.add_system(update_alien_animations.run_in_state(GameState::LoadWaveState));
        fixedupdate.add_system(check_for_alien_collisions.run_in_state(GameState::Playing));

        app.add_stage_before(
            CoreStage::Update,
            "Alien Fixed Timestep",
            FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate),
        )
        .add_startup_system(load_assets)
        .add_system_to_stage(CoreStage::PostUpdate, Rylo::update_position_tweens)
        .add_enter_system(GameState::LoadWaveState, spawn_aliens);
    }
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Sprites>,
    mut animations: ResMut<Animations>,
) {
    sprites.add("ALIEN_BULLET", asset_server.load("images/alien_bullet/bullet.png"));
    sprites.add("ALIEN_BULLET_FLASH", asset_server.load("images/alien_bullet/bullet_flash.png"));
    sprites.add("ALIEN_WALK_1", asset_server.load("images/alien_ferris/walk_1.png"));
    sprites.add("ALIEN_WALK_2", asset_server.load("images/alien_ferris/walk_2.png"));
    sprites.add("RYLO_ALIEN", asset_server.load("images/unsafe_ferris_2.png"));
    sprites.add("RYLO_BULLET_FLASH", asset_server.load("images/rylo_bullet_flash.png"));
    sprites.add("RYLO_BULLET", asset_server.load("images/rylo_bullet.png"));

    let alien_animation = Animation::from_images(
        vec!["ALIEN_WALK_1".to_string(), "ALIEN_WALK_2".to_string()],
        ALIEN_WALK_FRAME_DURATION_IN_MILLIS 
    ); 
    animations.add("ALIEN_WALK", alien_animation);
}

#[derive(Bundle)]
struct AlienBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
    alien: Alien,
    shooting_cooldown: ShootingCooldown,
}

#[derive(Bundle)]
struct ArisAlienBundle {
    #[bundle]
    alien_bundle: AlienBundle,
    aris: Aris,
    velocity: Velocity,
    name: Name,
}

#[derive(Bundle)]
struct RyloAlienBundle {
    #[bundle]
    alien_bundle: AlienBundle,
    rylo: Rylo,
    name: Name,
}

impl AlienBundle {
    fn new(translation: Vec2, size: Vec2, texture: Handle<Image>, cooldown: DurationType) -> AlienBundle {
        AlienBundle {
            alien: Alien,
            sprite_bundle: SpriteBundle {
                transform: Transform { translation: translation.extend(0.0), ..default() },
                sprite: Sprite { custom_size: Some(size), ..default() },
                texture,
                ..default()
            },
            collider: Collider { size },
            shooting_cooldown: ShootingCooldown::new(cooldown),
        }
    }

    fn new_aris(translation: Vec2, velocity: Vec2, texture: Handle<Image>) -> ArisAlienBundle {
        ArisAlienBundle {
            name: Name::new("Aris"), 
            aris: Aris,
            alien_bundle: AlienBundle::new(
                translation, 
                Aris::SIZE, 
                texture, 
                DurationType::AtMost(AtMost(Aris::MAX_SHOOTING_COOLDOWN_IN_SECONDS))
            ),
            velocity: Velocity(velocity),
        }
    }

    fn new_rylo(translation: Vec2, texture: Handle<Image>) -> RyloAlienBundle {
        RyloAlienBundle {
            name: Name::new("Rylo"),
            rylo: Rylo,
            alien_bundle: AlienBundle::new(
                translation, 
                Rylo::SIZE, 
                texture, 
                DurationType::AtMost(AtMost(Rylo::MAX_SHOOTING_COOLDOWN_IN_SECONDS))
            ),
        }
    }
}

fn wave_zero(mut commands: Commands, animations: Res<Animations>, sprites: Res<Sprites>) {
    let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP.x + Aris::SIZE.x / 2.;
    let first_alien_y = TOP_WALL - ALIEN_WALL_GAP.y - Aris::SIZE.y / 2. - 80.;

    let total_alien_width = Aris::SIZE.x + ALIEN_ALIEN_GAP.x;
    let total_alien_height = Aris::SIZE.y + ALIEN_ALIEN_GAP.y;

    // spawn aliens
    for row in 0..5 {
        if row == 2 {
            continue;
        }
        for col in 0..(9 + row % 2) {
            if (row == 1 || row == 3) && col % 3 == 0 {
                continue;
            }
            let alien_x = first_alien_x + col as f32 * total_alien_width
                - ALIEN_ODD_ROW_OFFSET * ((row % 2) as f32);
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
                    Vec2::new(starting_x, starting_y),
                    Vec2::new(Aris::SPEED * Aris::INITIAL_DIRECTION, 0.0),
                    sprites.get("ALIEN_WALK_1") 
                ))
                .insert_bundle(AnimationBundle::from_animation(animations.get("ALIEN_WALK")))
                .insert(Animator::new(position_tween));
        }
    }
}

fn wave_one(mut commands: Commands, animations: Res<Animations>, sprites: Res<Sprites>) {
    let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP.x + Aris::SIZE.x / 2.;
    let first_alien_y = TOP_WALL - ALIEN_WALL_GAP.y - Aris::SIZE.y / 2. - 80.;

    let total_alien_width = Aris::SIZE.x + ALIEN_ALIEN_GAP.x;
    let total_alien_height = Aris::SIZE.y + ALIEN_ALIEN_GAP.y;

    // spawn aliens
    for row in 0..5 {
        if row == 2 {
            continue;
        }
        for col in 0..(9 + row % 2) {
            if (row == 0 || row == 4) && col % 3 == 0 {
                continue;
            }
            let alien_x = first_alien_x + col as f32 * total_alien_width
                - ALIEN_ODD_ROW_OFFSET * ((row % 2) as f32);
            let alien_y = first_alien_y - row as f32 * total_alien_height;

            let starting_x = LEFT_WALL + (random::<f32>() * WINDOW_WIDTH);
            let starting_y = BOTTOM_WALL + WINDOW_HEIGHT / 2.0 + (random::<f32>() * WINDOW_HEIGHT);

            if random::<f32>() > 0.8 {
                let position_tween = Rylo::get_position_tween(
                    Vec2::new(starting_x, starting_y), 
                    Vec2::new(alien_x, alien_y), 
                    LOAD_WAVE_DURATION_IN_SECONDS + 4.0
                ); 

                commands
                    .spawn()
                    .insert_bundle(AlienBundle::new_rylo(
                        Vec2::new(alien_x, alien_y),
                        sprites.get("RYLO_ALIEN"),
                    ))
                    .insert(Animator::new(position_tween))
                    .insert(AnimationState::default());
            } else {
                let position_tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    TweeningType::Once,
                    duration_between(0.25, LOAD_WAVE_DURATION_IN_SECONDS),
                    TransformPositionLens {
                        start: Vec3::new(starting_x, starting_y, 0.0),
                        end: Vec3::new(alien_x, alien_y, 0.0),
                    },
                );
    
                commands
                .spawn()
                .insert_bundle(AlienBundle::new_aris(
                    Vec2::new(starting_x, starting_y),
                    Vec2::new(Aris::SPEED * Aris::INITIAL_DIRECTION, 0.0),
                    sprites.get("ALIEN_WALK_1") 
                ))
                .insert_bundle(AnimationBundle::from_animation(animations.get("ALIEN_WALK")))
                .insert(Animator::new(position_tween));
            }

        }
    }
}

fn spawn_aliens(
    commands: Commands,
    animations: Res<Animations>,
    sprites: Res<Sprites>,
    global: Res<Global>,
) {
    match global.current_wave() {
        0 => wave_zero(commands, animations, sprites),
        // 0 => wave_one(commands, animations, sprites),
        1 => wave_one(commands, animations, sprites),
        _ => panic!("Wave not implemented"),
    }
}

fn update_aris_aliens(
    mut alien_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut ShootingCooldown,
            &Collider,
        ),
        With<Aris>,
    >,
    mut commands: Commands,
    sprites: Res<Sprites>,
) {
    let alien_forward_shift = ALIEN_ALIEN_GAP.y / 2. + Aris::SIZE.y / 2.;

    for (alien_entity, mut transform, mut velocity, mut shooting_cooldown, collider) in
        &mut alien_query
    {
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
            shooting_cooldown.reset();

            let offset = if random::<f32>() < 0.5 { 1.0 } else { -1.0 };
            let bullet_x = transform.translation.x + offset * Aris::SIZE.x / 2.;
            let bullet_y = transform.translation.y - Aris::SIZE.y / 4.;

            commands.spawn().insert_bundle(BulletBundle::from_aris(
                Vec2::new(bullet_x, bullet_y),
                sprites.get("ALIEN_BULLET"),
            ));

            let bullet_flash = commands
                .spawn()
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec2::new(offset * Aris::SIZE.x / 2., 0.).extend(1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(BULLET_FLASH_SIZE),
                        ..default()
                    },
                    texture: sprites.get("ALIEN_BULLET_FLASH"),
                    ..default()
                })
                .insert(DespawnTimer::from_seconds(BULLET_FLASH_DURATION_IN_SECONDS))
                .id();

            commands.entity(alien_entity).add_child(bullet_flash);
        } 
    }
}

fn check_for_alien_collisions(
    mut scoreboard: ResMut<Scoreboard>,
    alien_query: Query<(Entity, &Transform, &Collider, Option<&Rylo>, Option<&Aris>), With<Alien>>,
    bullet_query: Query<(Entity, &Bullet, &Transform, &Collider)>,
    animations: Res<Animations>,
    mut commands: Commands,
) {
    for (alien_entity, transform, alien_collider, maybe_rylo, maybe_aris) in &alien_query {
        for (bullet_entity, bullet, bullet_transform, bullet_collider) in &bullet_query {
            if bullet == &Bullet::Alien {
                // ignore bullets from other aliens
                continue;
            }
            if let Some(_collision) = collide(
                transform.translation,
                alien_collider.size,
                bullet_transform.translation,
                bullet_collider.size,
            ) {
                commands.entity(bullet_entity).despawn_recursive();
                commands.entity(alien_entity).despawn_recursive();

                let explosion = animations.get("EXPLOSION");
                let texture_atlas = match &explosion.image_data {
                    ImageData::TextureAtlas(texture_atlas) => texture_atlas,
                    _ => panic!("Explosion is stored as a texture atlas!"),
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

                if maybe_rylo.is_some() {
                    scoreboard.score += Rylo::DESTROY_ALIEN_SCORE;
                } else if maybe_aris.is_some() {
                    scoreboard.score += DESTROY_ALIEN_SCORE;
                }
                break;
            }
        }
    }
}

fn update_alien_animations(
    mut query: Query<(&mut AnimationState, &BAnimation, &mut Handle<Image>), With<Alien>>,
    sprites: Res<Sprites>,
    animations: Res<Animations>,
) {
    let alien_walk_animation = animations.get("ALIEN_WALK");
    let images = match &alien_walk_animation.image_data {
        ImageData::Images(images) => images,
        _ => panic!("Image data not found"),
    };

    for (mut animation_state, alien_animation, mut texture) in query.iter_mut() {
        animation_state.update(alien_animation, Duration::from_secs_f32(TIME_STEP));
        *texture = sprites.get(images[animation_state.frame_index() as usize].as_str())
    }
}
