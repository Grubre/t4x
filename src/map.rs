use noise::{NoiseFn, OpenSimplex};

#[derive(Debug, Clone)]
pub enum TileType {
    Plains,
    Desert,
    Hills,
}

#[derive(Clone)]
pub enum UnitType {
    Civilian,
    Builder,
}

#[derive(Clone)]
pub struct Unit {
    pub unit_type: UnitType,
    pub position: (u64, u64),
}

#[derive(Clone)]
pub struct Building {}

#[derive(Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub unit: Option<Unit>,
    pub building: Option<Building>,
}

pub fn generate_map(width: u16, height: u16) -> Vec<Vec<Tile>> {
    let scale = 20.0;
    let noise = OpenSimplex::new(1);
    let mut vec: Vec<_> = vec![vec![]; width as usize];
    for x in 0..width {
        for y in 0..height {
            let val = noise.get([f64::from(x) / scale, f64::from(y) / scale]);
            let tile_type = if val < 0.4 {
                TileType::Plains
            } else {
                TileType::Hills
            };
            vec[x as usize].push(Tile {
                tile_type,
                unit: None,
                building: None,
            });
        }
    }
    vec
}
