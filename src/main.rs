use std::{time::Duration, collections::HashMap};
use benimator::FrameRate;
use rand::*;

use bevy::{prelude::*, time::FixedTimestep, sprite::collide_aabb::collide};

const TIME_STEP: f32 = 1.0 / 60.0;
const CAMERA_LEVEL: f32 = 1.0;

// window 
const WINDOW_WIDTH: f32 = 920.;
const WINDOW_HEIGHT: f32 = 920.;

// background
const BACKGROUND_FONT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const BACKGROUND_FONT_SIZE: f32 = 30.0;
const BACKGROUND_LEVEL: f32 = -1.0;

// scoreboard
const SCORE_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const SCOREBOARD_FONT_SIZE: f32 = 30.0;
const SCOREBOARD_PADDING_TOP: Val = Val::Px(8.0);
const SCOREBOARD_PADDING_LEFT: Val = Val::Px(10.0);

// walls
const WALL_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const BOTTOM_WALL: f32 = -WINDOW_HEIGHT / 2.;
const TOP_WALL: f32 = WINDOW_HEIGHT / 2.;
const LEFT_WALL: f32 = -WINDOW_WIDTH / 2.;
const RIGHT_WALL: f32 = WINDOW_WIDTH / 2.;

const WALL_THICKNESS: f32 = 10.;

// ship 
const SHIP_BULLET_IMAGE_SIZE: Vec2 = Vec2::new(512.0, 512.0);
const SHIP_IMAGE_SIZE: Vec2 = Vec2::new(1200.0, 800.0);
const SHIP_SIZE: Vec2 = Vec2::new(120., 80.);
const GAP_BETWEEN_SHIP_AND_FLOOR: f32 = 5.0;
const SHIP_SPEED: f32 = 450.;
const SHOOTING_COOLDOWN_IN_SECONDS: f32 = 0.8;
const SHIP_BULLET_SIZE: Vec2 = Vec2::new(40.0, 40.0);

// bullet 
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);
const SHIP_BULLET_SPEED: f32 = 450.0;
const SHIP_BULLET_INITIAL_GAP: f32 = 5.;

// alien
const ALIEN_IMAGE_SIZE: Vec2 = Vec2::new(1200.0, 800.0);
const ALIEN_BULLET_COLOR: Color = Color::rgb(0.0, 0.9, 0.0);
const ALIEN_ODD_ROW_OFFSET: f32 = 30.0;
const ALIEN_WALL_GAP: f32 = 20.;
const ALIEN_SIZE: Vec2 = Vec2::new(81., 54.);
const ALIEN_SPEED: f32 = 75.;
const ALIEN_ALIEN_GAP: Vec2 = Vec2::new(30., 50.);
const ALIEN_BULLET_SPEED: f32 = 300.0;
const INITIAL_ALIEN_DIRECTION: f32 = 1.; // right
const DESTROY_ALIEN_SCORE: u32 = 5;
const MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS: f32 = 15.;

// explosion
const EXPLOSION_SIZE: f32 = 0.3;
const EXPLOSION_FRAME_DURATION_IN_MILLIS: u64 = 20;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum GameState {
    Playing,
    GameOver,
}

#[derive(Component)]
struct Background;

struct Scoreboard { score: u32, }

#[derive(Component)]
struct Collider;

#[derive(Component, Deref, DerefMut)]
struct ShootingCooldown(Timer);

#[derive(Component)]
struct Ship;

#[derive(Component, PartialEq)]
enum Bullet {
    Ship,
    Alien
}

#[derive(Component)]
struct Alien;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Explosion;

#[derive(Default, Component, Deref, DerefMut)]
struct AnimationState(benimator::State);

#[derive(Component, Deref, Clone)]
struct BAnimation(benimator::Animation);

// TODO: use readonly public crate
struct Animation {
    pub animation: BAnimation,
    pub texture_atlas: Handle<TextureAtlas>   
}

struct Animations {
    animations: HashMap<String, Animation>
}

impl Animations {
    pub fn new() -> Self {
        Animations {
            animations: HashMap::default()
        }
    }
    pub fn add(&mut self, animation_name: String, animation: Animation) {
        self.animations.insert(animation_name, animation);
    }
    pub fn get(&self, animation_name: String) -> Option<&Animation> {
        self.animations.get(&animation_name)
    }
}


struct Sprites {
    sprites: HashMap<String, Handle<Image>>
}

impl Sprites {
    pub fn new() -> Self {
        Sprites {
            sprites: HashMap::default()
        }
    }
    pub fn add(&mut self, sprite_name: String, sprite: Handle<Image>) {
        self.sprites.insert(sprite_name, sprite);
    }
    pub fn get(&self, sprite_name: String) -> Option<&Handle<Image>> {
        self.sprites.get(&sprite_name)
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Turbo".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: false,
            ..default()
        })
        .insert_resource(Animations::new())
        .insert_resource(Sprites::new())
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        // .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Scoreboard { score: 0 })
        .add_startup_system(setup)
        .add_state(GameState::Playing)
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update_ship.before(update_bullets))
                .with_system(update_aliens.before(update_bullets))
                .with_system(update_bullets)
                .with_system(check_for_alien_collisions.after(update_bullets))
                .with_system(check_gameover.after(check_for_alien_collisions))
        )
        .add_system_set(
            SystemSet::on_enter(GameState::GameOver)
                .with_system(play_gameover)
        )
        .add_system(update_scoreboard)
        .add_system(update_explosions)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}


#[derive(Bundle)]
struct BulletBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    bullet: Bullet,
    collider: Collider,
    velocity: Velocity
}

impl BulletBundle {
    fn from_alien(translation: Vec2) -> BulletBundle {
        BulletBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: BULLET_SIZE.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: ALIEN_BULLET_COLOR,
                    ..default()
                },
                ..default()
            },
            velocity: Velocity(Vec2::new(0.0, -ALIEN_BULLET_SPEED)),
            bullet: Bullet::Alien,
            collider: Collider,
        }
    }

    fn from_ship(translation: Vec2, sprite: Handle<Image>) -> BulletBundle {
        BulletBundle {
            sprite_bundle: SpriteBundle {
                texture: sprite,
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: SHIP_BULLET_SIZE.extend(1.0),
                    ..default()
                },
                sprite: generate_texture_sprite(SHIP_BULLET_SIZE, SHIP_BULLET_IMAGE_SIZE), 
                ..default()
            },
            velocity: Velocity(Vec2::new(0.0, SHIP_BULLET_SPEED)),
            bullet: Bullet::Ship,
            collider: Collider,
        }
    }
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(WINDOW_WIDTH, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Animations>,
    mut sprites: ResMut<Sprites>
) {
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, CAMERA_LEVEL),
            ..default()
        },
        ..default()
    });

    // background
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, BACKGROUND_LEVEL),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
                ..default()
            },
            texture: asset_server.load("images/space.png"),
            ..default() 
        });

     // spawn scoreboard
     commands.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            }),
        ]) 
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_PADDING_TOP, 
                left: SCOREBOARD_PADDING_LEFT,
                ..default()
            },
            ..default()
        }),
    );

    // background
    commands
        .spawn()
        .insert_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "DEFEAT THE BORROW CHECKER",
                    TextStyle {
                        font_size: BACKGROUND_FONT_SIZE,
                        color: BACKGROUND_FONT_COLOR,
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    },
                ),
            ]) 
            .with_text_alignment(TextAlignment::CENTER)
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(Background);


    // animations
    let explosion_atlas = TextureAtlas::from_grid(
        asset_server.load("images/explosion_sheet.png"),
        Vec2::new(100.0, 100.0),
        5, // columns
        5, // rows
    );
    let explosion_animation = Animation {
        animation: BAnimation(benimator::Animation::from_indices(
            0..25,
            FrameRate::from_frame_duration(Duration::from_millis(EXPLOSION_FRAME_DURATION_IN_MILLIS)),
        )),
        texture_atlas: texture_atlases.add(explosion_atlas)
    };

    animations.add("EXPLOSION".to_string(), explosion_animation);

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
    
    let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP + ALIEN_SIZE.x / 2.;
    let first_alien_y = TOP_WALL - ALIEN_WALL_GAP  - ALIEN_SIZE.y / 2.;

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
                        scale: ALIEN_SIZE.extend(1.0),
                        ..default()
                    },
                    sprite: generate_texture_sprite(ALIEN_SIZE, ALIEN_IMAGE_SIZE), 
                    texture: asset_server.load("images/alien_ferris.png"),
                    ..default()
                })
                .insert(ShootingCooldown(Timer::from_seconds(random::<f32>() * MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS, false)))
                .insert(Velocity(Vec2::new(ALIEN_SPEED * INITIAL_ALIEN_DIRECTION, 0.0)))
                .insert(Collider);
        }
    }

    // spawn walls
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Left));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Right));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Top));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Bottom));

    // bullets
    sprites.add("FERRIS_BULLET".to_string(), asset_server.load("images/rust_white.png"))
}

fn generate_texture_sprite(entity_size: Vec2, texture_size: Vec2) -> Sprite {
    let size_x = entity_size.x / texture_size.x * 10.0;
    let size_y = entity_size.y / texture_size.y * 10.0;

    Sprite {
        custom_size: Some(Vec2::new(size_x, size_y) * Vec2::splat(1.4)),
        ..default()
    }
}

fn update_explosions(
    mut query: Query<(Entity, &mut AnimationState, &mut TextureAtlasSprite, &BAnimation), With<Explosion>>,
    mut commands: Commands
) {
    for (explosion_entity, mut animation_state, mut texture_atlas, explosion_animation) in query.iter_mut() {
        if animation_state.frame_index() == 24 {
            // TODO: .is_ended() _should_ work?!
            // if animation_state.is_ended() {
            commands.entity(explosion_entity).despawn();
        }
        animation_state.update(explosion_animation, Duration::from_secs_f32(TIME_STEP));
        texture_atlas.index = animation_state.frame_index();
    }
}

fn check_gameover(
    alien_query: Query<&Transform, With<Alien>>, 
    bullet_query: Query<(&Transform, &Bullet)>,
    ship_query: Query<&Transform, With<Ship>>,
    mut game_state: ResMut<State<GameState>>
) {
    if game_state.current() == &GameState::GameOver {
        return;
    }

    if alien_query.is_empty() {
        game_state.set(GameState::GameOver).unwrap(); 
        return;
    }

    let ship_transform = ship_query.single();

    for alien_transform in &alien_query {
        if alien_transform.translation.y < ship_transform.translation.y {
            game_state.set(GameState::GameOver).unwrap(); 
            return;
        }
    }

    for (bullet_transform, bullet) in &bullet_query {
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
            game_state.set(GameState::GameOver).unwrap(); 
            break;
        } 
    }
}

fn play_gameover() {
    println!("Game Over!")
}

fn check_for_alien_collisions(
    mut scoreboard: ResMut<Scoreboard>,
    alien_query: Query<(Entity, &Transform), With<Alien>>, 
    bullet_query: Query<(Entity, &Bullet, &Transform)>,
    animations: Res<Animations>,
    mut commands: Commands,
) {
    for (alien_entity, transform) in &alien_query {
        for (bullet_entity, bullet, bullet_transform) in &bullet_query {
            if bullet == &Bullet::Alien {
                // ignore bullets from other aliens
                continue;
            }
            if let Some(_collision) = collide(
                transform.translation,
                transform.scale.truncate(),
                bullet_transform.translation,
                bullet_transform.scale.truncate(),
            ) {
                commands.entity(bullet_entity).despawn();
                commands.entity(alien_entity).despawn();

                let explosion = animations.get("EXPLOSION".to_string()).unwrap();

                commands
                    .spawn()
                    .insert_bundle(SpriteSheetBundle {
                        texture_atlas: explosion.texture_atlas.clone(),
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

fn update_aliens(
    mut alien_query: Query<(&mut Transform, &mut Velocity, &mut ShootingCooldown), With<Alien>>,
    mut commands: Commands
) {
    let alien_forward_shift = ALIEN_ALIEN_GAP.y / 2. + ALIEN_SIZE.y / 2.;

    for (mut transform, mut velocity, mut shooting_cooldown) in &mut alien_query {
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
            let bullet_x;
        
            // randomly shoot from left or right extent
            if random::<f32>() < 0.5 {
                bullet_x = transform.translation.x + ALIEN_SIZE.x / 2.;
            } else {
                bullet_x = transform.translation.x - ALIEN_SIZE.x / 2.;
            } 

            let bullet_y = transform.translation.y - ALIEN_SIZE.y / 2.; 

            commands
                .spawn()
                .insert_bundle(BulletBundle::from_alien(Vec2::new(bullet_x, bullet_y)));

            shooting_cooldown.reset();
            shooting_cooldown.set_duration(Duration::from_secs_f32(random::<f32>() * MAX_ALIEN_SHOOTING_COOLDOWN_IN_SECONDS))
        } else {
            shooting_cooldown.tick(Duration::from_secs_f32(TIME_STEP));
        }
    } 
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, Without<Background>>) {
    let mut score_text = query.single_mut();
    score_text.sections[1].value = scoreboard.score.to_string();
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

fn update_bullets(mut bullet_query: Query<(&mut Transform, &Velocity), With<Bullet>>) {
    for (mut transform, velocity) in &mut bullet_query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}
