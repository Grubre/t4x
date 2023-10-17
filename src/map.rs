#[derive(Debug, Clone)]
pub enum TileType {
    Plains,
    Desert,
}

#[derive(Clone)]
pub struct Unit {}

#[derive(Clone)]
pub struct Building {}

#[derive(Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub unit: Option<Unit>,
    pub building: Option<Building>,
}

pub fn generate_map(width: u16, height: u16) -> Vec<Vec<Tile>> {
    let mut vec: Vec<_> = vec![vec![]; width as usize];
    for x in 0..width {
        for y in 0..height {
            vec[x as usize].push(Tile{ tile_type: TileType::Plains, unit: None, building: None });
        }
    }
    vec
}

