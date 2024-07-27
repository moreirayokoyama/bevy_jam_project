use bevy::prelude::*;
use noise::utils::NoiseMap;
use rand::distributions::Standard;
use rand::prelude::*;

use crate::{BLOCK_SIZE, WORLD_WIDTH};

#[derive(Resource)]
pub struct GameWorld {
    pub width: i32,
    pub noise_map: NoiseMap,
    surface_height: Vec<f32>,
}

impl GameWorld {
    pub fn new(noise_map: NoiseMap, surface_height: Vec<f32>) -> GameWorld {
        GameWorld {
            width: WORLD_WIDTH as i32,
            noise_map,
            surface_height,
        }
    }

    pub fn get_height_in_blocks(&self, x: usize) -> f32 {
        let height = self.surface_height[x % WORLD_WIDTH].trunc();
        let left_height = if x > 0 {
            self.surface_height[x - 1].trunc()
        } else {
            self.surface_height[WORLD_WIDTH - 1].trunc()
        };
        let right_height = if x < (WORLD_WIDTH - 1) {
            self.surface_height[x + 1].trunc()
        } else {
            self.surface_height[0].trunc()
        };

        if height > left_height && height > right_height {
            left_height.max(right_height)
        } else {
            height
        }
    }

    pub fn get_block_position(x: usize, y: usize) -> Vec2 {
        Vec2::new(
            ((x * BLOCK_SIZE) as f32).trunc(),
            ((y * BLOCK_SIZE) as f32).trunc(),
        )
    }

    pub fn get_random_x_block(&self) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..self.width) as usize
    }

    pub fn get_surface(&self, x: usize) -> f32 {
        (self.get_height_in_blocks(x) * (BLOCK_SIZE as f32)) - ((BLOCK_SIZE / 2) as f32).trunc()
    }
}
