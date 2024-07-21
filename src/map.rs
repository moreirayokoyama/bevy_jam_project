use bevy::{
    app::{FixedUpdate, Plugin, Startup, Update}, asset::AssetServer, color::Color, hierarchy::{BuildChildren, DespawnRecursiveExt}, math::{Vec2, Vec3}, prelude::{default, Bundle, Commands, Component, Entity, IntoSystemConfigs, Query, Res, Resource, SpatialBundle, With}, sprite::{Sprite, SpriteBundle}, transform::{commands, components::Transform}
};
use noise::{
    core::perlin::perlin_2d,
    permutationtable::PermutationTable,
    utils::{NoiseMap, PlaneMapBuilder},
};

use crate::{
    control::CurrentControlOffset, utils, BLOCK_SIZE, BLOCK_Y_COUNT, CANVAS_WIDTH, CHUNKS_TO_LOAD, CHUNK_INITIAL_OFFSET, CHUNK_WIDTH, FLOOR_MEDIAN, FLOOR_THRESHOLD, PIXEL_PERFECT_LAYERS, RES_HEIGHT_OFFSET, WORLD_WIDTH
};

const MAX_OFFSET: f32 = ((CHUNKS_TO_LOAD * CHUNK_WIDTH * BLOCK_SIZE)/2) as f32;

#[derive(Resource)]
struct GameWorld {
    noise_map: NoiseMap,
    surface_height: Vec<f64>,
}

#[derive(Resource)]
struct CurrentChunkOffset(usize);

#[derive(Component)]
struct Chunk {
    id: usize,
    x_offset: f32,
    y_offset: f32,
}

#[derive(PartialEq)]
enum Block {
    Air,
    Solid(SolidBlock),
}

#[derive(PartialEq)]
enum SolidBlock {
    Surface,
    Stone,
    Earth,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, (initialize, startup).chain())
            .add_systems(Update, map_movement)
        ;
    }
}

fn initialize(mut commands: Commands) {
    let noise_map = generate_noise_map();
    let surface_height = generate_surface_height_vec(&noise_map);

    commands.insert_resource(GameWorld {
        noise_map,
        surface_height,
    });
    commands.insert_resource(CurrentChunkOffset(CHUNK_INITIAL_OFFSET));
}

fn startup(
    mut commands: Commands,
    game_world: Res<GameWorld>,
    current_chunk_offset: Res<CurrentChunkOffset>,
    asset_server: Res<AssetServer>,
) {
    let half_chunks_to_load = CHUNKS_TO_LOAD as i32 / 2;
    let remaining_chunks_to_load = CHUNKS_TO_LOAD as i32 % 2;
    
    for i in -half_chunks_to_load..(half_chunks_to_load + remaining_chunks_to_load)  {
        let chunk_id = ((current_chunk_offset.0 as i32) + i) as usize;
        let start_x = CHUNK_WIDTH * chunk_id;
        let start_y: usize = 0;

        let canvas_x: f32 = (i * (CHUNK_WIDTH * BLOCK_SIZE) as i32) as f32;

        //criação de um chunk
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(canvas_x, RES_HEIGHT_OFFSET as f32, 2.),
                ..default()
            }, 
            Chunk {
                id: chunk_id,
                x_offset: canvas_x,
                y_offset: RES_HEIGHT_OFFSET as f32,
            })
        ).with_children(|parent| {
            for col_x in 0..CHUNK_WIDTH {
                for col_y in 0..BLOCK_Y_COUNT {
                    let x = start_x + col_x;
                    let y = start_y + col_y;

                    match get_block(x, y, &game_world) {
                        Block::Air => {},
                        Block::Solid(SolidBlock::Earth) => { 
                            parent.spawn((new_earth_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); 
                        },
                        Block::Solid(SolidBlock::Stone) => { parent.spawn((new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); },
                        Block::Solid(SolidBlock::Surface) => { 
                            let v = (new_surface_block(col_x, col_y), PIXEL_PERFECT_LAYERS);
                            parent.spawn(v); 
                        },
                    }
                }
            }
        });
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::linear_rgb(0., 0.5, 0.0),
                custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                (-4 * (BLOCK_SIZE as i32)) as f32,
                (4 * BLOCK_SIZE) as f32,
                2.,
            )),
            ..default()
        },
        PIXEL_PERFECT_LAYERS,
    ));
}

fn new_earth_block(x: usize, y: usize) -> SpriteBundle {
    new_block(x, y, Color::linear_rgb(0.34375, 0.22265, 0.15234))
}

fn new_stone_block(x: usize, y: usize) -> SpriteBundle {
    new_block(x, y, Color::linear_rgb(0.5, 0.5, 0.5))
}

fn new_surface_block(x: usize, y: usize) -> SpriteBundle {
    new_block(x, y, Color::linear_rgb(0.0, 0.35, 0.0))
}

fn new_block(x: usize, y: usize, color: Color) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            color: color,
            custom_size: Some(Vec2::new(
                BLOCK_SIZE as f32,
                BLOCK_SIZE as f32,
            )),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(
            (x * BLOCK_SIZE) as f32,
            (y * BLOCK_SIZE) as f32,
            2.,
        )),
        ..default()
    }
}

fn map_movement(mut query: Query<(Entity, &mut Transform, &mut Chunk)>, control_offset: Res<CurrentControlOffset>, mut commands: Commands, game_world: Res<GameWorld>) {
    for (entity, mut transform, mut chunk) in query.iter_mut() {
        transform.translation.x -= control_offset.0;
        transform.translation.y -= control_offset.1;
        chunk.x_offset -= control_offset.0;
        chunk.y_offset -= control_offset.1;

        if (chunk.x_offset > MAX_OFFSET && control_offset.0 < 0.) || (chunk.x_offset < -MAX_OFFSET && control_offset.0 > 0.) {
            let new_chunk_offset = if chunk.x_offset > 0. { chunk.x_offset-MAX_OFFSET*2. } else { chunk.x_offset+MAX_OFFSET*2. };
            let chunk_id = if chunk.x_offset > 0. { chunk.id - CHUNKS_TO_LOAD } else { chunk.id + CHUNKS_TO_LOAD };
            let start_x = CHUNK_WIDTH * chunk_id;
            let start_y: usize = 0;

            commands
            .spawn((SpatialBundle {
                transform: Transform::from_xyz(new_chunk_offset, chunk.y_offset, 2.),
                ..default()
            }, Chunk {
                id: chunk_id,
                x_offset: new_chunk_offset,
                y_offset: chunk.y_offset,
            }))
            .with_children(|parent| {
                for col_x in 0..CHUNK_WIDTH {
                    for col_y in 0..BLOCK_Y_COUNT {
                        let x = start_x + col_x;
                        let y = start_y + col_y;
                        
                        match get_block(x, y, &game_world) {
                            Block::Air => {},
                            Block::Solid(SolidBlock::Earth) => { parent.spawn((new_earth_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); },
                            Block::Solid(SolidBlock::Stone) => { parent.spawn((new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); },
                            Block::Solid(SolidBlock::Surface) => { parent.spawn((new_surface_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); },
                        }
                    }
                }
            });
            commands.entity(entity).despawn_recursive();

            
        }
    }
}

fn get_block(x: usize, y: usize, game_world: &GameWorld) -> Block {
    if (y as f64) < game_world.surface_height[x] && (y + 1) as f64 >= game_world.surface_height[x] {
        Block::Solid(SolidBlock::Surface)
    } else if (y as f64) < game_world.surface_height[x] {
        Block::Solid(SolidBlock::Earth)
    } else {
        Block::Air
    }
}

fn generate_noise_map() -> NoiseMap {
    let hasher = PermutationTable::new(0);
    let r = PlaneMapBuilder::new_fn(|point| perlin_2d(point.into(), &hasher))
        .set_size(WORLD_WIDTH, 1)
        .set_x_bounds(-32., 32.)
        .set_y_bounds(-32., 32.)
        .build();

    utils::write_example_to_file(&r, "world.png");
    r
}

fn generate_surface_height_vec(noise_map: &NoiseMap) -> Vec<f64> {
    let mut v = Vec::<f64>::with_capacity(WORLD_WIDTH);
    for x in 0..WORLD_WIDTH {
        v.push(FLOOR_MEDIAN + noise_map.get_value(x as usize, 0) * FLOOR_THRESHOLD);
    }
    v
}
