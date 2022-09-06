use std::time::Duration;

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use rand::random;

use crate::{shared::{reset_game, WINDOW_WIDTH, BOTTOM_WALL, LEFT_WALL, Velocity, DespawnTimer, WINDOW_HEIGHT, TIME_STEP, Sprites}, GameState, Scoreboard};

#[derive(Component)]
pub struct HallOfFame;

pub struct HallOfFamePlugin;

impl Plugin for HallOfFamePlugin {
    fn build(&self, app: &mut App) {
        let mut fixedupdate = SystemStage::parallel();
        fixedupdate.add_system_set(
            ConditionSet::new()
                .label("Hall Of Fame Updates")
                .run_in_state(GameState::Victory)
                .with_system(HallOfFame::update)
                .into(),
        );

        app.add_enter_system(GameState::Victory, reset_game.before(HallOfFame::create))
            .add_startup_system(HallOfFame::load_assets)
            .add_enter_system(GameState::Victory, HallOfFame::create)
            .add_exit_system(GameState::Victory, HallOfFame::cleanup)
            .add_stage_before(
                CoreStage::Update,
                "Hall Of Fame Fixed Timestep",
                FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate),
            );
    }
}

impl HallOfFame {
    fn load_assets(asset_server: Res<AssetServer>, mut sprites: ResMut<Sprites>) {
        sprites.add("HOF_FERRIS", asset_server.load("images/ferris.png"));

        for i in 0..=8 {
            sprites.add(format!("HOF_BULLET_{}", i).as_str(), asset_server.load(&format!("images/victory_bullets/{}.png", i))); 
        }
    }

    fn create(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        scoreboard: Res<Scoreboard>,
        sprites: Res<Sprites>
    ) {
        commands
            .spawn()
            .insert(HallOfFame)
            .insert(Name::new("Hall of Fame"))
            .insert_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "YOU STAND VICTORIOUS!\n\n",
                    TextStyle {
                        font_size: 70.0,
                        color: Color::rgb(1.0, 1.0, 0.0),
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    },
                ),
                TextSection::new(
                    format!("Maximum Score Reached: {}\n\n", scoreboard.score),
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    },
                ),
                TextSection::new(
                    "[Space] Bask in Glory\n[M] Menu\n[ESC] Quit",
                    TextStyle {
                        font_size: 30.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    },
                ),
            ])
            .with_style(Style {
                position_type: PositionType::Relative,
                margin: UiRect {
                    top: Val::Auto,
                    left: Val::Auto,
                    right: Val::Auto,
                    bottom: Val::Percent(4.0),
                },
                align_self: AlignSelf::Center,
                ..default()
            }),
        );

        commands
            .spawn()
            .insert(HallOfFame)
            .insert(Name::new("Hall of Fame Ferris"))
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 100.0, 0.0),
                    rotation: Quat::from_rotation_z(0.0),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(500.0, 2.0 / 3.0 * 500.0)),
                    ..default()
                },
                texture: sprites.get("HOF_FERRIS"),
                ..default()
            });
    }

    fn update(
        mut commands: Commands, 
        keyboard_input: Res<Input<KeyCode>>, 
        mut bullet_query: Query<(&mut Transform, &Velocity), With<HallOfFame>>,
        sprites: Res<Sprites>
    ) {
        let bask_in_glory = keyboard_input.pressed(KeyCode::Space);
        let menu = keyboard_input.pressed(KeyCode::M);

        if menu {
            commands.insert_resource(NextState(GameState::Menu));
        } else if bask_in_glory {
            for _ in 0..(20.0 * random::<f32>()) as i32 {
                 let bullet_x = LEFT_WALL + WINDOW_WIDTH * random::<f32>(); 
                 let bullet_y = BOTTOM_WALL - 100.0; 

                 let velocity = Vec2::new(
                    0.0,
                    f32::max(250.0, WINDOW_HEIGHT * (0.5 + random::<f32>())), 
                );

                let bullet_path = format!("HOF_BULLET_{}", (random::<f32>() * 9.0) as usize);

                commands
                    .spawn()
                    .insert(HallOfFame)
                    .insert_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(bullet_x, bullet_y, 0.0), 
                            scale: Vec3::new(2.0, 1.0, 1.0),
                            rotation: Quat::from_rotation_z(180.0f32.to_radians()),
                            ..default()
                        },
                        texture: sprites.get(bullet_path.as_str()),
                        sprite: Sprite {
                            flip_x: true,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(DespawnTimer::from_seconds(3.0))
                    .insert(Velocity(velocity));
            }
        }

        for (mut transform, velocity) in &mut bullet_query {
            transform.translation.x += velocity.x * TIME_STEP;
            transform.translation.y += velocity.y * TIME_STEP;
        }
    }

    fn cleanup(mut commands: Commands, query: Query<Entity, With<HallOfFame>>) {
        for hall_of_fame_entity in query.iter() {
            commands.entity(hall_of_fame_entity).despawn();
        } 
    }
}



