use map::Tile;

pub mod display;
pub mod map;

pub struct State {
    pub pointer_pos: (u64, u64),
    pub tiles: Vec<Vec<Tile>>,
}


