use bevy::prelude::*;
use noise::utils::NoiseMap;

use crate::WORLD_WIDTH;

#[derive(Resource)]
pub struct GameWorld {
    pub noise_map: NoiseMap,
    surface_height: Vec<f32>,
}

impl GameWorld {
    pub fn new(noise_map: NoiseMap, surface_height: Vec<f32>) -> GameWorld {
        GameWorld {
            noise_map,
            surface_height,
        }
    }

    pub fn get_height(&self, x: usize) -> f32 {
        self.surface_height[x % WORLD_WIDTH]
    }
}
