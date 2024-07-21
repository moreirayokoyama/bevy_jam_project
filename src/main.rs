// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//TODO: Carregar mais chunks (o suficiente pra preencher todo o canvas + 2)
//TODO: Despawn dos chunks mais distantes
//TODO: receber algum input e usá-lo pra forçar um offset dos chunks
//TODO: Ponderar sobre tamanho do bloco, tamanho do chunk, tamanho do mundo

mod utils;
mod camera;
mod map;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use camera::CameraPlugin;
use map::MapPlugin;

pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

pub const RES_WIDTH: usize = 768;
pub const RES_HEIGHT: usize = 432;
//const RES_WIDTH_OFFSET: usize = -(RES_WIDTH / 2);
pub const RES_HEIGHT_OFFSET: i32 = -((RES_HEIGHT as i32) / 2);

pub const BLOCK_SIZE: usize = 16;

pub const BLOCK_X_COUNT: usize = RES_WIDTH / BLOCK_SIZE;
pub const BLOCK_Y_COUNT: usize = RES_HEIGHT / BLOCK_SIZE;

pub const FLOOR_MEDIAN: f64 = (BLOCK_Y_COUNT as f64) * 0.5;
pub const FLOOR_THRESHOLD: f64 = FLOOR_MEDIAN * 0.5;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }), CameraPlugin, MapPlugin))
        .run();
}

