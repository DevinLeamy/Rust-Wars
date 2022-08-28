use super::{bounds::Bounds, tile_set::TileSet};
use bevy::prelude::*;

/// client-specific resource used to build a level
pub struct LevelOptions {
    pub bounds: Bounds,
    pub tile_set: TileSet,
    pub tile_size: f32,
}
