use bevy_tweening::{lens::TransformPositionLens, *};
use rand::random;
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

impl Alien {
    pub fn position_tween(start: Vec2, end: Vec2, duration: DurationType) -> Tween<Transform> {
        Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            duration.sample(),
            TransformPositionLens {
                start: start.extend(0.0), 
                end: end.extend(0.0) 
            },
        ) 
    }
}

#[derive(Component)]
pub struct Aris;

impl Aris {
    pub const SIZE: Vec2 = Vec2::new(60.0, 40.);
    pub const SPEED: f32 = 75.;
    pub const BULLET_SPEED: f32 = 300.0;
    pub const MAX_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 10.;
    pub const INITIAL_DIRECTION: f32 = 1.; // right
    pub const LAYER: f32 = 0.5;
}

#[derive(Component)]
pub struct Zorg;

impl Zorg {
    pub const SIZE: Vec2 = Vec2::new(70.0, 60.);
    pub const MAX_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 15.0;
    pub const SCORE_VALUE: u32 = 20;
    pub const LAYER: f32 = 0.0;
    pub const BULLET_SIZE: Vec2 = Vec2::new(30.0, 60.0);
    pub const BULLET_SPEED: f32 = 150.0;
}

impl Zorg {
    fn update(
        mut query: Query<(Entity, &mut Transform, &mut ShootingCooldown), With<Zorg>>,
        mut commands: Commands,
        sprites: Res<Sprites>
    ) {
        for (entity, transform, mut cooldown) in query.iter_mut() {
            if !cooldown.finished() {
                continue;
            }
            cooldown.reset();

            Zorg::shoot(
                entity, 
                transform.translation.truncate(), 
                Vec2::new(-0.3 * Zorg::BULLET_SPEED, -0.7 * Zorg::BULLET_SPEED),
                &mut commands,
                &sprites
            );

            Zorg::shoot(
                entity, 
                transform.translation.truncate(), 
                Vec2::new(0.0 * Zorg::BULLET_SPEED, -1.0 * Zorg::BULLET_SPEED),
                &mut commands,
                &sprites
            );

            Zorg::shoot(
                entity, 
                transform.translation.truncate(), 
                Vec2::new(0.3 * Zorg::BULLET_SPEED, -0.7 * Zorg::BULLET_SPEED),
                &mut commands,
                &sprites
            );
        }

    }

    fn shoot(zorg_entity: Entity, origin: Vec2, velocity: Vec2, commands: &mut Commands, sprites: &Res<Sprites>) {
        let rotation;
        if velocity.x == 0.0 {
            rotation = 0.0
        } else {
            let angle = 90.0 - f32::atan(velocity.y.abs() / velocity.x.abs()).to_degrees();
            rotation = angle * velocity.x.signum();
        }

        commands.spawn().insert_bundle(BulletBundle::from_zorg(
            origin,
            sprites.get("ZORG_BULLET"),
            Velocity(velocity),
            rotation
        ));

        let bullet_flash = commands
            .spawn()
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec2::new(0.0, 0.0).extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(BULLET_FLASH_SIZE),
                    ..default()
                },
                texture: sprites.get("ZORG_BULLET_FLASH"),
                ..default()
            })
            .insert(DespawnTimer::from_seconds(BULLET_FLASH_DURATION_IN_SECONDS))
            .id();

        commands.entity(zorg_entity).add_child(bullet_flash); 
    }
}

#[derive(Component)]
pub struct Rylo;

impl Rylo {
    const SIZE: Vec2 = Vec2::new(40.0, 40.);
    pub const BULLET_SPEED: f32 = 300.0;
    const SCORE_VALUE: u32 = 10;
    const MAX_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 10.0;
    const POSITION_TWEEN_COMPLETE: u64 = 1;
    const POSITION_DELAY_COMPLETE: u64 = 2;
    const POSITION_TWEEN_DELAY: f32 = 10.0;
    const POSITION_TWEEN_DURATION: f32 = 10.0;
    pub const LAYER: f32 = 0.3;
}

impl Rylo {
    fn position_tween(start: Vec2, end: Vec2, completion_id: u64, duration: DurationType) -> Tween<Transform> {
        Alien::position_tween(start, end, duration).with_completed_event(completion_id)
    }

    fn update(
        mut query: Query<(Entity, &mut Transform, &mut ShootingCooldown), With<Rylo>>,
        mut commands: Commands,
        sprites: Res<Sprites>
    ) {
        for (alien_entity, transform, mut cooldown) in query.iter_mut() {
            if !cooldown.finished() {
                continue;
            }
            cooldown.reset();

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
        }
   }

    fn update_position_tweens(mut commands: Commands, query: Query<&Transform, With<Rylo>>, mut event_reader: EventReader<TweenCompleted>) {
        for event in event_reader.iter() {
            let entity = event.entity;
            let tween_id = event.user_data;

            if tween_id != Rylo::POSITION_TWEEN_COMPLETE && tween_id != Rylo::POSITION_DELAY_COMPLETE {
                continue;
            }

            let alien_transform = query.get(entity).expect("rylo not found");
            let translation = alien_transform.translation.truncate();

            let tween = match tween_id {
                Rylo::POSITION_TWEEN_COMPLETE => {
                    Rylo::position_tween(
                        translation, 
                        translation, 
                        Rylo::POSITION_DELAY_COMPLETE,
                        DurationType::Between(Between(1., Rylo::POSITION_TWEEN_DELAY))
                    )
                },
                Rylo::POSITION_DELAY_COMPLETE => {
                    let ending_x = LEFT_WALL + (random::<f32>() * WINDOW_WIDTH);
                    let ending_y = BOTTOM_WALL + WINDOW_HEIGHT / 2.0 + (random::<f32>() * WINDOW_HEIGHT / 2.0);
    
                    Rylo::position_tween(
                        alien_transform.translation.truncate(), 
                        Vec2::new(ending_x, ending_y),
                        Rylo::POSITION_TWEEN_COMPLETE,
                        DurationType::Between(Between(1., Rylo::POSITION_TWEEN_DURATION)) 
                    )
                },
                _ => panic!("INVALID TWEEN ID")
            };

            commands.entity(entity).insert(Animator::new(tween));
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
                .with_system(Zorg::update)
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
    sprites.add("ARIS_ALIEN", asset_server.load("images/alien_ferris.png"));
    sprites.add("ZORG_ALIEN", asset_server.load("images/robot_ferris.png"));
    sprites.add("ZORG_BULLET_FLASH", asset_server.load("images/zorg_bullet_flash.png"));
    sprites.add("ZORG_BULLET", asset_server.load("images/zorg_bullet.png"));

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

#[derive(Bundle)]
struct ZorgAlienBundle {
    #[bundle]
    alien_bundle: AlienBundle,
    zorg: Zorg,
    name: Name,
}

impl AlienBundle {
    fn new(translation: Vec3, size: Vec2, texture: Handle<Image>, cooldown: DurationType) -> AlienBundle {
        AlienBundle {
            alien: Alien,
            sprite_bundle: SpriteBundle {
                transform: Transform { translation, ..default() },
                sprite: Sprite { custom_size: Some(size), ..default() },
                texture,
                ..default()
            },
            collider: Collider { size },
            shooting_cooldown: ShootingCooldown::new(cooldown),
        }
    }

    fn new_aris(translation: Vec2, velocity: Vec2, sprites: &Res<Sprites>) -> ArisAlienBundle {
        ArisAlienBundle {
            name: Name::new("Aris"), 
            aris: Aris,
            alien_bundle: AlienBundle::new(
                translation.extend(Aris::LAYER), 
                Aris::SIZE, 
                sprites.get("ARIS_ALIEN"), 
                DurationType::AtMost(AtMost(Aris::MAX_SHOOTING_COOLDOWN_IN_SECONDS))
            ),
            velocity: Velocity(velocity),
        }
    }

    fn new_rylo(translation: Vec2, sprites: &Res<Sprites>) -> RyloAlienBundle {
        RyloAlienBundle {
            name: Name::new("Rylo"),
            rylo: Rylo,
            alien_bundle: AlienBundle::new(
                translation.extend(Rylo::LAYER), 
                Rylo::SIZE, 
                sprites.get("RYLO_ALIEN"), 
                DurationType::AtMost(AtMost(Rylo::MAX_SHOOTING_COOLDOWN_IN_SECONDS))
            ),
        }
    }

    fn new_zorg(translation: Vec2, sprites: &Res<Sprites>) -> ZorgAlienBundle {
        ZorgAlienBundle {
            name: Name::new("Zorg"),
            zorg: Zorg,
            alien_bundle: AlienBundle::new(
                translation.extend(Zorg::LAYER), 
                Zorg::SIZE, 
                sprites.get("ZORG_ALIEN"), 
                DurationType::AtMost(AtMost(Zorg::MAX_SHOOTING_COOLDOWN_IN_SECONDS))
            ),
        }
    }
}

fn spawn_aliens(
    mut commands: Commands,
    animations: Res<Animations>,
    sprites: Res<Sprites>,
    global: Res<Global>,
) {
    match global.current_wave() {
        0 => Wave::load_from_file("assets/waves/wave_0.txt").initialize(commands, sprites, animations),
        1 => Wave::load_from_file("assets/waves/wave_1.txt").initialize(commands, sprites, animations),
        2 => Wave::load_from_file("assets/waves/wave_2.txt").initialize(commands, sprites, animations),
        3 => Wave::load_from_file("assets/waves/wave_3.txt").initialize(commands, sprites, animations),
        4 => commands.insert_resource(NextState(GameState::Victory)),
        _ => panic!("Invalid Wave!")
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
    alien_query: Query<(Entity, &Transform, &Collider, Option<&Rylo>, Option<&Aris>, Option<&Zorg>), With<Alien>>,
    bullet_query: Query<(Entity, &Bullet, &Transform, &Collider)>,
    animations: Res<Animations>,
    mut commands: Commands,
) {
    for (alien_entity, transform, alien_collider, maybe_rylo, maybe_aris, maybe_zorg) in &alien_query {
        for (bullet_entity, bullet, bullet_transform, bullet_collider) in &bullet_query {
            if bullet == &Bullet::Alien {
                continue;
            }

            if collide(
                transform.translation,
                alien_collider.size,
                bullet_transform.translation,
                bullet_collider.size,
            ).is_some() {
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
                    .insert_bundle(AnimationBundle::from_animation(explosion))
                    .insert(Explosion);

                if maybe_rylo.is_some() { scoreboard.score += Rylo::SCORE_VALUE; } 
                else if maybe_aris.is_some() { scoreboard.score += DESTROY_ALIEN_SCORE; }
                else if maybe_zorg.is_some() { scoreboard.score += Zorg::SCORE_VALUE; }

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

pub struct Wave {
    layout: Vec<Vec<char>>
}

impl Wave {
    pub fn load_from_file(path: &str) -> Wave {
        let wave_data = std::fs::read_to_string(path).expect("Wave file not found!");
        let lines: Vec<&str> = wave_data.split_whitespace().collect();
        
        let mut layout = vec![vec![' '; lines[0].len()]; lines.len()];

        for (i, line) in lines.iter().enumerate() {
            let line: Vec<char> = line.as_bytes().iter().map(|c| c.to_ascii_lowercase() as char).collect();
            for j in 0..line.len() as usize {
                layout[i][j] = line[j];
            }
        }

        Wave { layout }
    }

    pub fn initialize(&self, mut commands: Commands, sprites: Res<Sprites>, animations: Res<Animations>) {
        for row in 0..self.layout.len() {
            for col in 0..self.layout[0].len() {
                let alien_type = self.layout[row][col];

                match alien_type {
                    '#' => continue,
                    'a' => Wave::initialize_aris(&sprites, &animations, &mut commands, row as u32, col as u32),
                    'r' => Wave::initialize_rylo(&sprites, &mut commands, row as u32, col as u32), 
                    'z' => Wave::initialize_zorg(&sprites, &mut commands, row as u32, col as u32),
                    _   => panic!("INVALID WAVE LAYOUT") 
                }
            }
        }
    }

    pub fn get_translation(row: u32, col: u32) -> Vec2 {
        let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP.x + Aris::SIZE.x / 2.;
        let first_alien_y = TOP_WALL - ALIEN_WALL_GAP.y - Aris::SIZE.y / 2. - 80.;
    
        let total_alien_width = Aris::SIZE.x + ALIEN_ALIEN_GAP.x;
        let total_alien_height = Aris::SIZE.y + ALIEN_ALIEN_GAP.y;

        let alien_x = first_alien_x + col as f32 * total_alien_width - ALIEN_ODD_ROW_OFFSET * ((row % 2) as f32);
        let alien_y = first_alien_y - row as f32 * total_alien_height;

        Vec2::new(alien_x, alien_y)
    }

    pub fn get_starting_location() -> Vec2 {
        Vec2::new(
            LEFT_WALL + (random::<f32>() * WINDOW_WIDTH),
            BOTTOM_WALL + WINDOW_HEIGHT / 2.0 + (random::<f32>() * WINDOW_HEIGHT)
        ) 
    }

    fn initialize_aris(sprites: &Res<Sprites>, animations: &Res<Animations>, commands: &mut Commands, row: u32, col: u32) {
        let starting_translation = Wave::get_starting_location(); 
        let ending_translation = Wave::get_translation(row, col);

        let position_tween = Alien::position_tween(
            starting_translation,
            ending_translation,
            DurationType::Between(Between(0.25, LOAD_WAVE_DURATION_IN_SECONDS))
        );

        commands
            .spawn()
            .insert_bundle(AlienBundle::new_aris(
                starting_translation,
                Vec2::new(Aris::SPEED * Aris::INITIAL_DIRECTION, 0.0),
                sprites 
            ))
            .insert_bundle(AnimationBundle::from_animation(animations.get("ALIEN_WALK")))
            .insert(Animator::new(position_tween));
    }

    fn initialize_rylo(sprites: &Res<Sprites>, commands: &mut Commands, row: u32, col: u32) {
        let starting_translation = Wave::get_starting_location(); 
        let ending_translation = Wave::get_translation(row, col);

        let position_tween = Rylo::position_tween(
            starting_translation,
            ending_translation,
            Rylo::POSITION_TWEEN_COMPLETE,
            DurationType::Fixed(Fixed(LOAD_WAVE_DURATION_IN_SECONDS + 2.))
        ); 

        commands
            .spawn()
            .insert_bundle(AlienBundle::new_rylo(
                starting_translation,
                sprites,
            ))
            .insert(Animator::new(position_tween));
    }

    fn initialize_zorg(sprites: &Res<Sprites>, commands: &mut Commands, row: u32, col: u32) {
        let starting_translation = Wave::get_starting_location(); 
        let ending_translation = Wave::get_translation(row, col);

        let position_tween = Alien::position_tween(
            starting_translation,
            ending_translation,
            DurationType::Between(Between(0.25, LOAD_WAVE_DURATION_IN_SECONDS))
        );

        commands
            .spawn()
            .insert_bundle(AlienBundle::new_zorg(
                starting_translation,
                sprites,
            ))
            .insert(Animator::new(position_tween));
    }
}
