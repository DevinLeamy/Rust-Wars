use std::time::Duration;

use bevy::{prelude::*, time::FixedTimestep, sprite::collide_aabb::collide};

const TIME_STEP: f32 = 1.0 / 60.0;
const BACKGROUND_COLOR: Color = Color::BEIGE;

// window 
const WINDOW_WIDTH: f32 = 920.;
const WINDOW_HEIGHT: f32 = 920.;

// scoreboard
const SCORE_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
const SCOREBOARD_FONT_SIZE: f32 = 30.0;
const SCOREBOARD_PADDING_TOP: Val = Val::Px(8.0);
const SCOREBOARD_PADDING_LEFT: Val = Val::Px(10.0);

// walls
const WALL_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const BOTTOM_WALL: f32 = -WINDOW_HEIGHT / 2.;
const TOP_WALL: f32 = WINDOW_HEIGHT / 2.;
const LEFT_WALL: f32 = -WINDOW_WIDTH / 2.;
const RIGHT_WALL: f32 = WINDOW_WIDTH / 2.;

const WALL_THICKNESS: f32 = 20.;

// player
const SHIP_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const SHIP_SIZE: Vec2 = Vec2::new(80., 20.);
const GAP_BETWEEN_SHIP_AND_FLOOR: f32 = 20.;
const SHIP_SPEED: f32 = 200.;

// bullet 
const BULLET_COLOR: Color = Color::rgb(0.9, 0.0, 0.0);
const BULLET_SIZE: Vec2 = Vec2::new(7.0, 20.0);
const BULLET_SPEED: f32 = 600.;
const SHIP_BULLET_INITIAL_GAP: f32 = 10.;
const SHOOTING_COOLDOWN_IN_SECONDS: f32 = 0.8;

// alien
const ALIEN_COLOR: Color = Color::rgb(0.0, 0.8, 0.0);
const ALIEN_WALL_GAP: f32 = 20.;
const ALIEN_SIZE: Vec2 = Vec2::new(60., 20.);
const ALIEN_SPEED: f32 = 125.;
const ALIEN_ALIEN_GAP: Vec2 = Vec2::new(30., 50.);
const INITIAL_ALIEN_DIRECTION: f32 = 1.; // right
const DESTROY_ALIEN_SCORE: u32 = 5;

struct Scoreboard { score: u32, }

#[derive(Component)]
struct Collider;

#[derive(Component, Deref, DerefMut)]
struct ShootingCooldown(Timer);

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Alien;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Turbo".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Scoreboard { score: 0 })
        .add_startup_system(setup)
        // .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update_ship)
                .with_system(update_bullets)
                .with_system(move_aliens)
                .with_system(check_for_alien_collisions)
                // .with_system(move_paddle.before(check_for_collisions))
                // .with_system(apply_velocity.before(check_for_collisions))
                // .with_system(check_for_collisions)
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

     // spawn scoreboard
     commands.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
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

    let ship_y = BOTTOM_WALL + GAP_BETWEEN_SHIP_AND_FLOOR + SHIP_SIZE.y / 2.;

    // spawn player
    commands
        .spawn()
        .insert(Ship)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: SHIP_COLOR,
                ..default()
            }, 
            transform: Transform {
                translation: Vec3::new(0.0, ship_y, 0.0),
                scale: SHIP_SIZE.extend(1.),
                ..default()
            },
            ..default()
        })
        .insert(Collider);
    
    let first_alien_x = LEFT_WALL + ALIEN_WALL_GAP + ALIEN_SIZE.x / 2.;
    let first_alien_y = TOP_WALL - ALIEN_WALL_GAP  - ALIEN_SIZE.y / 2.;

    let total_alien_width = ALIEN_SIZE.x + ALIEN_ALIEN_GAP.x;
    let total_alien_height = ALIEN_SIZE.y + ALIEN_ALIEN_GAP.y;
    
    // spawn aliens
    for row in 0..5 {
        for col in 0..5 {
            let alien_x = first_alien_x + col as f32 * total_alien_width; 
            let alien_y = first_alien_y - row as f32 * total_alien_height; 
            commands
                .spawn()
                .insert(Alien)
                .insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: ALIEN_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(alien_x, alien_y, 0.0),
                        scale: ALIEN_SIZE.extend(1.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(Velocity(Vec2::new(ALIEN_SPEED * INITIAL_ALIEN_DIRECTION, 0.0)))
                .insert(Collider);
        }
    }

    // spawn walls
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Left));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Right));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Top));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Bottom));

   
}

fn check_for_alien_collisions(
    mut scoreboard: ResMut<Scoreboard>,
    alien_query: Query<(Entity, &Transform), With<Alien>>, 
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    mut commands: Commands,
) {

    for (alien, transform) in &alien_query {
        for (bullet, bullet_transform) in &bullet_query {
            if let Some(_collision) = collide(
                transform.translation,
                transform.scale.truncate(),
                bullet_transform.translation,
                bullet_transform.scale.truncate(),
            ) {
                commands.entity(bullet).despawn();
                commands.entity(alien).despawn();

                scoreboard.score += DESTROY_ALIEN_SCORE;
                break;
            }
        }
    }
}

fn move_aliens(mut alien_query: Query<(&mut Transform, &mut Velocity), With<Alien>>) {
    let alien_forward_shift = ALIEN_ALIEN_GAP.y / 2. + ALIEN_SIZE.y / 2.;

    for (mut transform, mut velocity) in &mut alien_query {
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
    } 
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut score_text = query.single_mut();
    score_text.sections[1].value = scoreboard.score.to_string();
}

fn update_ship(
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<(Entity, &mut Transform, Option<&mut ShootingCooldown>), With<Ship>>, 
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

    // update cooldown timer
    if let Some(cooldown_timer) = &mut shooting_cooldown {
        if cooldown_timer.finished() {
            commands.entity(ship).remove::<ShootingCooldown>();
        } else {
            cooldown_timer.tick(Duration::from_secs_f32(TIME_STEP));
        }
    }

    if shoot && shooting_cooldown.is_none() {
        let bullet_x = transform.translation.x;
        let bullet_y = transform.translation.y + transform.scale.y / 2. + SHIP_BULLET_INITIAL_GAP;

        commands
            .entity(ship)
            .insert(ShootingCooldown(Timer::from_seconds(SHOOTING_COOLDOWN_IN_SECONDS, false)));

        commands
            .spawn()
            .insert(Bullet)
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: BULLET_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(bullet_x, bullet_y, 0.),
                    scale: BULLET_SIZE.extend(1.),
                    ..default()
                },
                ..default()
            })
            .insert(Velocity(Vec2::new(0., BULLET_SPEED)))
            .insert(Collider);
    }
}

fn update_bullets(mut bullet_query: Query<(&mut Transform, &Velocity), With<Bullet>>) {
    for (mut transform, velocity) in &mut bullet_query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}
