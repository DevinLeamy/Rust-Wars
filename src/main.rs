use std::{time::Duration};

use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use iyes_loopless::prelude::*;

mod player;
use player::{PlayerPlugin, Ship};

mod aliens;
use aliens::{Alien, AliensPlugin};

mod shared;
use shared::*;

mod debug;
use debug::DebugPlugin;

mod gameover;
use gameover::GameOverPlugin;

mod menu;
use menu::MenuPlugin;

mod hall_of_fame;
use hall_of_fame::HallOfFamePlugin;

mod waves;

const LOAD_WAVE_DURATION_IN_SECONDS: f32 = 3.0;

#[derive(Deref, DerefMut)]
struct LoadWaveTimer(Timer);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum GameState {
    Menu,          // Game menu (press space to play)
    Playing,       // Player and enemies can move and shoot
    GameOver,      // Player is frozen and enemies have been despawned (press r to restart)
    LoadWaveState, // Load enemies into the scene (player and enemies cannot shoot)
    Victory,       // You win!
}

#[derive(Component)]
struct Background;

#[derive(Component)]
pub struct Scoreboard {
    score: u32,
}

#[derive(Component)]
struct Explosion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub enum TurboStages {
    FixedUpdate,
}

pub struct Global {
    is_playing: bool,
    wave: Option<u32>,
}

impl Global {
    pub fn reset(&mut self) {
        self.is_playing = false;
        self.wave = Some(0);
    }

    pub fn current_wave(&self) -> u32 {
        self.wave.unwrap()
    }

    pub fn start_playing(&mut self) {
        self.is_playing = true;
        self.wave = Some(0);
    }

    pub fn wave_cleared(&mut self) {
        self.wave = Some(self.wave.unwrap() + 1);
    }

    pub fn stop_playing(&mut self) {
        self.is_playing = false;
        self.wave = None;
    }
}

fn main() {
    console_error_panic_hook::set_once();

    let mut fixedupdate = SystemStage::parallel();
    fixedupdate.add_system(update_bullets.run_in_state(GameState::Playing));
    fixedupdate.add_system(check_gameover.run_in_state(GameState::Playing));
    fixedupdate.add_system(update_shooting_cooldowns.run_in_state(GameState::Playing));
    fixedupdate.add_system(update_load_wave.run_in_state(GameState::LoadWaveState));
    fixedupdate.add_system(update_timed);

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Turbo".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: false,
            // position: WindowPosition::Centered(MonitorSelection::Number(0)),
            canvas: Some("#bevy".to_owned()),
            ..default()
        })
        .add_loopless_state(GameState::Menu)
        .add_plugins(DefaultPlugins)
        // startup
        .add_startup_system(load_assets_and_animations.before(setup))
        .add_startup_system(setup)
        // fixed update
        .add_stage_before(
            CoreStage::Update,
            "Main Fixed TimeStep",
            FixedTimestepStage::from_stage(Duration::from_secs_f32(TIME_STEP), fixedupdate),
        )
        // resources
        .insert_resource(Animations::new())
        .insert_resource(Sprites::new())
        // plugins
        .add_plugin(TweeningPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(GameOverPlugin)
        .add_plugin(HallOfFamePlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(AliensPlugin)
        .add_plugin(AnimationPlugin::default())
        .add_system(check_wave_end.run_in_state(GameState::Playing))
        .add_enter_system(
            GameState::LoadWaveState,
            setup_load_wave,
        )
        .add_system(update_scoreboard.run_in_state(GameState::Playing))
        .add_system(update_explosions)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn load_assets_and_animations(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Animations>,
    mut sprites: ResMut<Sprites>,
) {
    sprites.add("SPACE_BACKGROUND", asset_server.load("images/space.png"));

    let explosion_atlas = TextureAtlas::from_grid(
        asset_server.load("images/explosion_sheet.png"),
        Vec2::new(100.0, 100.0),
        5, // columns
        5, // rows
    );
    let explosion_animation = Animation::from_texture(
        texture_atlases.add(explosion_atlas),
        25,
        EXPLOSION_FRAME_DURATION_IN_MILLIS 
    );

    animations.add("EXPLOSION", explosion_animation);
}

fn setup(mut commands: Commands, sprites: Res<Sprites>) {
    commands.insert_resource(Global {
        is_playing: false,
        wave: None,
    });

    commands.spawn_bundle(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, CAMERA_LEVEL),
            ..default()
        },
        ..default()
    });

    // background
    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, BACKGROUND_LEVEL),
            ..default()
        },
        sprite: Sprite {
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        texture: sprites.get("SPACE_BACKGROUND"),
        ..default()
    }); 

    // spawn walls
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Left));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Right));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Top));
    commands.spawn().insert_bundle(WallBundle::new(WallLocation::Bottom));
}

fn update_explosions(
    mut query: Query<
        (
            Entity,
            &mut AnimationState,
            &mut TextureAtlasSprite,
            &BAnimation,
        ),
        With<Explosion>,
    >,
    mut commands: Commands,
) {
    for (explosion_entity, mut animation_state, mut texture_atlas, explosion_animation) in
        query.iter_mut()
    {
        if animation_state.frame_index() == 24 {
            // TODO: .is_ended() _should_ work?!
            // if animation_state.is_ended() {
            commands.entity(explosion_entity).despawn();
        }
        animation_state.update(explosion_animation, Duration::from_secs_f32(TIME_STEP));
        texture_atlas.index = animation_state.frame_index();
    }
}

fn check_gameover(ship_query: Query<&Health, With<Ship>>, game_state: Res<CurrentState<GameState>>, mut commands: Commands) {
    if game_state.as_ref() == &CurrentState(GameState::GameOver) {
        return;
    }

    let ship_health = ship_query.single();

    if ship_health.0 == 0 {
        commands.insert_resource(NextState(GameState::GameOver));
    }
}

fn check_wave_end(
    alien_query: Query<With<Alien>>, 
    bullet_query: Query<Entity, With<Bullet>>, 
    mut commands: Commands, 
    mut global: ResMut<Global>
) {
    if alien_query.is_empty() {
        for bullet in bullet_query.iter() {
            commands.entity(bullet).despawn();
        }
        global.wave_cleared();
        commands.insert_resource(NextState(GameState::LoadWaveState));

    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<Scoreboard>>) {
    let mut score_text = query.single_mut();
    score_text.sections[1].value = scoreboard.score.to_string();
}

fn update_bullets(mut bullet_query: Query<(&mut Transform, &Velocity), With<Bullet>>) {
    for (mut transform, velocity) in &mut bullet_query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn update_timed(mut commands: Commands, mut query: Query<(Entity, &mut DespawnTimer)>) {
    for (entity, mut despawn_timer) in query.iter_mut() {
        despawn_timer.tick(Duration::from_secs_f32(TIME_STEP));

        if despawn_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn setup_load_wave(
    mut commands: Commands, 
    global: ResMut<Global>,
    asset_server: Res<AssetServer>,
    scoreboard_query: Query<Entity, With<Scoreboard>>
) {
    commands.insert_resource(LoadWaveTimer(Timer::from_seconds(
        LOAD_WAVE_DURATION_IN_SECONDS + 2.0,
        false,
    )));

    if global.current_wave() != 0 {
        return;
    }
    for scoreboard in scoreboard_query.iter() { 
        commands.entity(scoreboard).despawn();
    }
    commands.remove_resource::<Scoreboard>();

    // spawn scoreboard
    commands
        .spawn()
        .insert_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "Score: ",
                    TextStyle {
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    },
                ),
                TextSection::new("0", TextStyle {
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
        ).insert(Scoreboard { score: 0 });
    commands.insert_resource(Scoreboard { score: 0 })
}

fn update_load_wave(mut commands: Commands, mut timer: ResMut<LoadWaveTimer>) {
    timer.tick(Duration::from_secs_f32(TIME_STEP));

    if timer.finished() {
        commands.insert_resource(NextState(GameState::Playing));
    }
}
