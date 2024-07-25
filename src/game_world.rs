use bevy::prelude::*;
use noise::utils::NoiseMap;

use crate::{BLOCK_SIZE, WORLD_WIDTH};

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

    pub fn get_height_in_blocks(&self, x: usize) -> f32 {
        self.surface_height[x % WORLD_WIDTH].trunc()
    }

    pub fn get_block_position(x: usize, y: usize) -> Vec2 {
        Vec2::new(
            ((x * BLOCK_SIZE) as f32).trunc(),
            ((y * BLOCK_SIZE) as f32).trunc(),
        )
    }

    pub fn get_surface(&self, x: usize) -> f32 {
        (self.get_height_in_blocks(x) * (BLOCK_SIZE as f32)) - ((BLOCK_SIZE / 2) as f32).trunc()
    }
}
