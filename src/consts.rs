pub const PI: f32 = std::f32::consts::PI;

pub const ANT_SIZE: f32 = 2.0;
pub const ANT_COLOR: &[f32; 4] = &[0.7, 0.7, 0.7, 1.0];
pub const ANT_COUNT: usize = 10000;
pub const WANDER_COEFFICIENT: f32 = 0.1;
pub const WALK_SPEED: f32 = 2.0;
pub const NEST_HONING_STRENGTH: f32 = 0.05;
pub const ANT_PHEROMONE_TIMER: usize = 60;

pub const GRID_SIZE: f32 = 10.0;
pub const GRID_COLORS: &[f32; 16] = &[
    0.0, 0.0, 0.0, 0.0, 1.0, 0.5, 1.0, 1.0, 0.5, 1.0, 0.5, 1.0, 0.2, 0.2, 0.2, 1.0,
];

pub const PHEROMONE_SIZE: f32 = 2.0;
pub const PHEROMONE_COLOR: &[f32; 4] = &[0.5, 1.0, 0.5, 1.0];
