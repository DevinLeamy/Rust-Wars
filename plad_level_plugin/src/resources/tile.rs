#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    FLOOR,
    EMPTY,
    GROUND,
}

impl Tile {
    fn is_solid(self) -> bool {
        matches!(self, Tile::GROUND)
    }
}
