// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//TODO: Ponderar sobre tamanho do bloco, tamanho do chunk, tamanho do mundo

mod camera;
mod map;
mod utils;
mod control;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use camera::CameraPlugin;
use control::ControlPlugin;
use map::MapPlugin;

pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

pub const CANVAS_WIDTH: usize = 768;
pub const CANVAS_HEIGHT: usize = 432;
//const RES_WIDTH_OFFSET: usize = -(RES_WIDTH / 2);
pub const RES_HEIGHT_OFFSET: i32 = -((CANVAS_HEIGHT as i32) / 2);

pub const BLOCK_SIZE: usize = 8;

pub const BLOCK_X_COUNT: usize = CANVAS_WIDTH / BLOCK_SIZE;
pub const BLOCK_Y_COUNT: usize = CANVAS_HEIGHT / BLOCK_SIZE;

pub const FLOOR_MEDIAN: f64 = (BLOCK_Y_COUNT as f64) * 0.5;
pub const FLOOR_THRESHOLD: f64 = FLOOR_MEDIAN * 0.5;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_COUNT: usize = WORLD_WIDTH / CHUNK_WIDTH;
pub const CHUNK_INITIAL_OFFSET: usize = CHUNK_COUNT / 2;

pub const CHUNKS_IN_CANVAS: usize = CANVAS_WIDTH / (CHUNK_WIDTH * BLOCK_SIZE);
pub const CHUNKS_LOAD_THRESHOLD: usize = 2;
pub const CHUNKS_TO_LOAD: usize = CHUNKS_IN_CANVAS + CHUNKS_LOAD_THRESHOLD;

pub const MOVEMENT_SPEED: usize = BLOCK_SIZE * 16;

pub const DAY_DURATION_IN_SECONDS: usize = 4 * 60;
pub const WORLD_WIDTH: usize = DAY_DURATION_IN_SECONDS * MOVEMENT_SPEED;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
            CameraPlugin,
            MapPlugin,
            ControlPlugin,
        ))
        .run();
}
