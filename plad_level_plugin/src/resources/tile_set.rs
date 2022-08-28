use super::tile::Tile;

#[derive(Clone)]
pub struct TileSet {
    pub tiles: Vec<Vec<Tile>>,
    pub rows: u32,
    pub cols: u32,
}

impl TileSet {
    pub fn new_empty(rows: u32, cols: u32) -> Self {
        let tiles = vec![vec![Tile::EMPTY; cols as usize]; rows as usize];

        TileSet { tiles, rows, cols }
    }

    fn is_valid_tile(&self, row: u32, col: u32) -> bool {
        !(row >= self.rows || col >= self.cols || row < 0 || col < 0)
    }

    pub fn set_tile(&mut self, row: u32, col: u32, new_tile: Tile) -> () {
        if self.is_valid_tile(row, col) {
            self.tiles[row as usize][col as usize] = new_tile;
        }
    }

    pub fn get_tile(&self, row: u32, col: u32) -> Option<Tile> {
        if self.is_valid_tile(row, col) {
            Some(self.tiles[row as usize][col as usize])
        } else {
            None
        }
    }
}
