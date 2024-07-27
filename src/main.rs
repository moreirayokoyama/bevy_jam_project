// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//TODO: Ponderar sobre tamanho do bloco, tamanho do chunk, tamanho do mundo
//TODO: Todo o sistema de coordenadas seja i32 começando em zero (somente valores positivos)

/**
 *
 * 20:26BigardiDEV: em resumo é um AABB com um for loop da posição antiga pra próxima posição pra evitar de passar por colisores quando tiver rapido
 * 20:27BigardiDEV: pra otimizar mete um spatial hashing baseado em grid que tá show
 * 20:28BigardiDEV: você tem lá seus grids que são "baldes" que seguram uma lista de entidades nele baseando na posição, aí cada entidade só precisa verificar a colisão com os baldes vizinhos, evita o big O notation
 */
mod camera;
mod character;
mod control;
mod game;
mod game_world;
mod map;
mod physics;
mod ui;
mod utils;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use game::GamePlugins;

use game_world::GameWorld;
use noise::{utils::*, Fbm, Worley};

pub const BACKGROUND_LAYERS: RenderLayers = RenderLayers::layer(0);
pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

pub const CANVAS_HEIGHT: usize = 432; //Canvas height in pixels
                                      //Canvas width based on height to keep 16:9 aspect ratio
pub const CANVAS_WIDTH: usize = CANVAS_HEIGHT / 9 * 16;

pub const BLOCK_SIZE: usize = 16; //block size in pixels (height and width)

pub const BLOCK_X_COUNT: usize = CANVAS_WIDTH / BLOCK_SIZE;
pub const BLOCK_Y_COUNT: usize = CANVAS_HEIGHT / BLOCK_SIZE;

pub const CHUNK_WIDTH: usize = 16; //chunk width in blocks
pub const CHUNK_COUNT: usize = WORLD_WIDTH / CHUNK_WIDTH;
pub const CHUNK_INITIAL_OFFSET: usize = CHUNK_COUNT / 2;

pub const CHUNKS_TO_LOAD: usize = 16;

pub const MAP_MOVEMENT_SPEED_IN_BLOCKS: usize = 4; //camera speed in blocks/second
pub const CAMERA_REGULAR_SPEED: usize = BLOCK_SIZE * MAP_MOVEMENT_SPEED_IN_BLOCKS; //camera speed in pixels/second

pub const CHARACTER_SIZE: usize = BLOCK_SIZE * 2;
pub const CHARACTER_MOVEMENT_SPEED: usize = BLOCK_SIZE * MAP_MOVEMENT_SPEED_IN_BLOCKS * 2; //camera speed in blocks/second
pub const CHARACTER_JUMP_SPEED: usize = CHARACTER_MOVEMENT_SPEED / 30; // * 5 / 2;
pub const CHARACTER_ROAMING_THRESHOLD: usize = CHUNK_WIDTH * BLOCK_SIZE;

pub const DAY_DURATION_IN_SECONDS: usize = 4 * 60;
pub const WORLD_WIDTH: usize = DAY_DURATION_IN_SECONDS * MAP_MOVEMENT_SPEED_IN_BLOCKS;
pub const WORLD_HEIGHT: usize = 128; //World height in blocks

pub const FLOOR_MEDIAN: f32 = (WORLD_HEIGHT as f32) * 0.5;
pub const FLOOR_THRESHOLD: f32 = FLOOR_MEDIAN * 0.5;
pub const WORLD_BOTTOM_OFFSET: i32 = -(WORLD_HEIGHT as i32 / 2);
pub const WORLD_BOTTOM_OFFSET_IN_PIXELS: i32 = WORLD_BOTTOM_OFFSET * BLOCK_SIZE as i32;
pub const WORLD_CENTER_COL: usize = (WORLD_WIDTH / 2) - 1;

const GRAVITY: f32 = -9.81;

fn main() {
    let noise_map = generate_noise_map();
    let surface_height = generate_surface_height_vec(&noise_map);

    App::new()
        .insert_resource(GameWorld::new(noise_map, surface_height))
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        resolution: (CANVAS_WIDTH as f32, CANVAS_HEIGHT as f32).into(),
                        ..default()
                    }),
                    ..default()
                }),
            GamePlugins,
        ))
        //bevy_rapier2d
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(20.0))
        //.add_plugins(RapierDebugRenderPlugin::default())
        //beby_inspector_egui
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

fn generate_noise_map() -> NoiseMap {
    let fbm = Fbm::<Worley>::new(0);
    let bounds = WORLD_WIDTH as f64 * 0.0025;
    let r = PlaneMapBuilder::new(fbm) //new_fn(|point| perlin_2d(point.into(), &hasher))
        .set_size(WORLD_WIDTH, 1)
        .set_x_bounds(-bounds * 1., bounds * 1.)
        .set_y_bounds(-bounds, bounds)
        .build();

    #[cfg(not(target_arch = "wasm32"))]
    utils::write_example_to_file(&r, "world.png");
    r
}

fn generate_surface_height_vec(noise_map: &NoiseMap) -> Vec<f32> {
    let mut v = Vec::<f32>::with_capacity(WORLD_WIDTH);
    for x in 0..WORLD_WIDTH {
        v.push(FLOOR_MEDIAN + noise_map.get_value(x as usize, 0) as f32 * FLOOR_THRESHOLD);
    }
    v
}
