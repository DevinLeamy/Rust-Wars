use super::tile::{Tile, Tile::*};
use super::tile_set::TileSet;

pub struct TileSetLoader;

impl TileSetLoader {
    pub fn load_tile_set(path: String) -> Option<TileSet> {
        let tiles: Vec<Vec<Tile>> = vec![
            vec![
                EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
            ],
            vec![
                EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
            ],
            vec![
                EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, FLOOR, EMPTY, EMPTY,
            ],
            vec![
                EMPTY, EMPTY, EMPTY, EMPTY, FLOOR, EMPTY, FLOOR, FLOOR, EMPTY, EMPTY,
            ],
            vec![
                FLOOR, FLOOR, FLOOR, FLOOR, FLOOR, FLOOR, FLOOR, FLOOR, FLOOR, FLOOR,
            ],
            vec![
                GROUND, GROUND, GROUND, GROUND, GROUND, GROUND, GROUND, GROUND, GROUND, GROUND,
            ],
        ];
        let tile_set = TileSet {
            rows: tiles.len() as u32,
            cols: tiles[0].len() as u32,
            tiles,
        };
        return Some(tile_set);
    }
}
