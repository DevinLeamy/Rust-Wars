use super::{bounds::Bounds, texture_map::TextureMap, tile_set::TileSet};

pub struct Level {
    pub bounds: Bounds,
    pub tile_set: TileSet,
    pub texture_map: TextureMap,
    pub tile_size: f32,
}

impl Level {}
