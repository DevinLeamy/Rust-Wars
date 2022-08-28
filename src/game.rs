use bevy::{
    core::FixedTimestep, input::mouse::mouse_button_input_system, prelude::*,
    render::view::RenderLayers,
};

use bevy_inspector_egui::{RegisterInspectable, WorldInspectorParams, WorldInspectorPlugin};
use plad_level_plugin::{Bounds, LevelOptions, LevelPlugin, TileSet, TileSetLoader};

use crate::utils::*; 
use crate::config::*; 
use crate::input::*; 
use crate::player::*;

use benimator::AnimationPlugin;

const BACKGROUND_COLOR: Color = Color::BEIGE;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    PLAYING,
}

pub struct Game {
    game_state: GameState,
    runner: App,
}

impl Game {
    pub fn new() -> Self {
        Self {
            game_state: GameState::PLAYING,
            runner: Game::initialize_app(),
        }
    }

    pub fn run(&mut self) -> () {
        self.runner.run();
    }

    fn add_plugins(app: &mut App) -> () {
        app.add_plugins(DefaultPlugins)
            .add_plugin(AnimationPlugin::default())
            .add_plugin(LevelPlugin)
            .add_plugin(WorldInspectorPlugin::new());
    }

    fn add_resources(app: &mut App) -> () {
        app.insert_resource(Msaa { samples: 1 })
            .insert_resource(MouseLocation(Vec2::new(0.0, 0.0)))
            .insert_resource(WorldInspectorParams {
                #[cfg(feature = "debug")]
                enabled: true,
                #[cfg(not(feature = "debug"))]
                enabled: false,
                ..default()
            })
            .insert_resource(ClearColor(BACKGROUND_COLOR));
    }

    fn add_universal_systems(app: &mut App) -> () {
        app.add_system(mouse_button_input_system)
            .add_system(bevy::input::system::exit_on_esc_system);
    }

    fn add_states(app: &mut App) -> () {
        app.add_state(GameState::PLAYING);
    }

    fn add_playing_state_systems(app: &mut App) -> () {
        app.add_system_set(
            SystemSet::on_update(GameState::PLAYING)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(mouse_position_system)
                .with_system(move_player_system)
                .with_system(horizontally_center_player_system),
        );
    }

    fn add_inspector_egui(app: &mut App) -> () {
        app.register_inspectable::<Tile>();
    }

    fn initialize_app() -> App {
        let mut app = App::new();
        // Note: WindowDescriptor must be added before anything else!
        app.insert_resource(WindowDescriptor {
            title: "Plad".to_string(),
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            resizable: false,
            ..default()
        });
        app.insert_resource(LevelOptions {
            bounds: Bounds {
                position: Vec2::new(WORLD_ZERO_X, WORLD_ZERO_Y),
                size: Vec2::new(WINDOW_HEIGHT as f32, WINDOW_WIDTH as f32),
            },
            tile_set: TileSetLoader::load_tile_set("placeholder".to_string()).unwrap(),
            tile_size: UNIT_WIDTH as f32,
        });
        Game::add_plugins(&mut app);
        Game::add_resources(&mut app);
        Game::add_startup_system(&mut app);
        Game::add_universal_systems(&mut app);
        Game::add_states(&mut app);
        Game::add_playing_state_systems(&mut app);
        Game::add_inspector_egui(&mut app);

        app
    }

    fn add_startup_system(app: &mut App) -> () {
        // app.add_startup_system(Game::startup_system);
        app.add_startup_system(initialize_player);
    }

    fn startup_system(mut commands: Commands) {
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    }
}
