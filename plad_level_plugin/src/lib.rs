use std::collections::HashMap;

use bevy::{asset::LoadState, prelude::*};

pub use components::*;
pub use resources::*;
pub use systems::*;

mod components;
mod resources;
mod systems;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(AppState::Setup);
        LevelPlugin::add_systems(app);
        LevelPlugin::add_resources(app);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Setup,
    Finished,
}

#[derive(Default)]
struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
}

impl LevelPlugin {
    fn add_resources(app: &mut App) {
        app.init_resource::<TileSpriteHandles>();
    }

    fn add_systems(app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Setup).with_system(LevelPlugin::load_tile_textures),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Setup).with_system(LevelPlugin::check_textures),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Finished).with_system(LevelPlugin::generate_level),
        );
    }

    fn build_texture_atlas(
        mut textures: ResMut<Assets<Image>>,
        tile_sprite_handles: Res<TileSpriteHandles>,
    ) -> TextureAtlas {
        let mut texture_atlas_builder = TextureAtlasBuilder::default();
        for handle in &tile_sprite_handles.handles {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Image>(), texture);
        }

        texture_atlas_builder.finish(&mut textures).unwrap()
    }

    fn get_texture_index(
        texture_atlas: &TextureAtlas,
        asset_server: &Res<AssetServer>,
        texture_path: &str,
    ) -> usize {
        let grass_texture = asset_server.get_handle(texture_path);
        texture_atlas.get_texture_index(&grass_texture).unwrap()
    }

    fn generate_level(
        mut commands: Commands,
        level_options: Res<LevelOptions>,
        tile_sprite_handles: Res<TileSpriteHandles>,
        asset_server: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        textures: ResMut<Assets<Image>>,
    ) {
        let texture_atlas = Self::build_texture_atlas(textures, tile_sprite_handles);

        let mut tile_index_map: HashMap<Tile, usize> = HashMap::new();
        tile_index_map.insert(
            Tile::GROUND,
            Self::get_texture_index(&texture_atlas, &asset_server, "textures/tiles/5.png"),
        );
        tile_index_map.insert(
            Tile::FLOOR,
            Self::get_texture_index(&texture_atlas, &asset_server, "textures/tiles/2.png"),
        );
        tile_index_map.insert(
            Tile::EMPTY,
            Self::get_texture_index(&texture_atlas, &asset_server, "textures/tiles/18.png"),
        );

        let atlas_handle = texture_atlases.add(texture_atlas);

        // create the tile shapes for all of the tiles
        let tile_set = level_options.tile_set.clone();
        let tile_size = level_options.tile_size;

        for i in 0..tile_set.rows as usize {
            for j in 0..tile_set.cols as usize {
                let row = tile_set.rows as usize - 1 - i;
                let col = tile_set.cols as usize - 1 - j;
                let tile = tile_set.get_tile(i as u32, j as u32).unwrap();

                let tile_sprite_bundle = SpriteSheetBundle {
                    transform: Self::generate_tile_transform(
                        row as u32,
                        col as u32,
                        &level_options.bounds,
                        tile_size,
                    ),
                    sprite: TextureAtlasSprite::new(*tile_index_map.get(&tile).unwrap()),
                    texture_atlas: atlas_handle.clone(),
                    ..default()
                };

                commands.spawn().insert_bundle(tile_sprite_bundle);
            }
        }

        commands.insert_resource(Level {
            bounds: level_options.bounds,
            tile_set,
            texture_map: TextureMap {},
            tile_size: level_options.tile_size,
        })
    }

    fn load_tile_textures(
        mut tile_sprite_handles: ResMut<TileSpriteHandles>,
        asset_server: Res<AssetServer>,
    ) {
        tile_sprite_handles.handles = asset_server.load_folder("textures/tiles").unwrap();
    }

    fn check_textures(
        mut state: ResMut<State<AppState>>,
        rpg_sprite_handles: ResMut<TileSpriteHandles>,
        asset_server: Res<AssetServer>,
    ) {
        if let LoadState::Loaded = asset_server
            .get_group_load_state(rpg_sprite_handles.handles.iter().map(|handle| handle.id))
        {
            state.set(AppState::Finished).unwrap();
        }
    }

    fn generate_tile_transform(
        grid_row: u32,
        grid_col: u32,
        level_bounds: &Bounds,
        tile_size: f32,
    ) -> Transform {
        let x = grid_col as f32 * tile_size + level_bounds.position.x;
        let y = grid_row as f32 * tile_size + level_bounds.position.y;
        let z = 4.0;

        Transform {
            translation: Vec3::new(x, y, z),
            // Note textures are displayed to individual tile_size specs
            scale: Vec3::ONE, // Vec3::new(tile_size, tile_size, 1.0),
            ..default()
        }
    }
}
