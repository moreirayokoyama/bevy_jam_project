use bevy::{
    app::{Plugin, Startup, Update}, asset::AssetServer, color::Color, hierarchy::{BuildChildren, DespawnRecursiveExt}, math::{Vec2, Vec3}, prelude::{default, Commands, Component, Entity, IntoSystemConfigs, Query, Res, Resource, SpatialBundle}, sprite::{Sprite, SpriteBundle}, transform::components::Transform
};

use crate::{
    control::CurrentControlOffset, utils, GameWorld, BLOCK_SIZE, CHUNKS_TO_LOAD, CHUNK_COUNT, CHUNK_INITIAL_OFFSET, CHUNK_WIDTH, FLOOR_MEDIAN, FLOOR_THRESHOLD, PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET, WORLD_HEIGHT, WORLD_WIDTH
};

const MAX_OFFSET: f32 = ((CHUNKS_TO_LOAD * CHUNK_WIDTH * BLOCK_SIZE)/2) as f32;

#[derive(Resource)]
struct CurrentChunkOffset(usize);

#[derive(Component)]
struct Chunk {
    id: usize,
    x_offset: f32,
    y_offset: f32,
}

#[derive(Component)]
pub struct Collider;

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
    commands.insert_resource(CurrentChunkOffset(CHUNK_INITIAL_OFFSET));
}

fn startup(
    mut commands: Commands,
    game_world: Res<GameWorld>,
    current_chunk_offset: Res<CurrentChunkOffset>,
    asset_server: Res<AssetServer>,
) {
    let _ = asset_server;
    let half_chunks_to_load = CHUNKS_TO_LOAD as i32 / 2;
    let remaining_chunks_to_load = CHUNKS_TO_LOAD as i32 % 2;
    
    for i in -half_chunks_to_load..(half_chunks_to_load + remaining_chunks_to_load)  {
        let chunk_id = ((current_chunk_offset.0 as i32) + i) as usize;
        let start_x = CHUNK_WIDTH * chunk_id;
        let start_y: usize = 0;

        let canvas_x: f32 = (i * (CHUNK_WIDTH * BLOCK_SIZE) as i32) as f32;
        let canvas_y: f32 = (WORLD_BOTTOM_OFFSET * BLOCK_SIZE as i32) as f32;

        //criação de um chunk
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(canvas_x, canvas_y, 2.),
                
                ..default()
            }, 
            Chunk {
                id: chunk_id,
                x_offset: canvas_x,
                y_offset: canvas_y,
            })
        ).with_children(|parent| {
            for col_x in 0..CHUNK_WIDTH {
                for col_y in 0..WORLD_HEIGHT {
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
            let next_index = if chunk.x_offset > 0. { chunk.id as i32 - CHUNKS_TO_LOAD as i32 } else { chunk.id as i32 + CHUNKS_TO_LOAD as i32 };
            let chunk_index = (next_index as usize % CHUNK_COUNT + CHUNK_COUNT) % CHUNK_COUNT;
            let start_x = CHUNK_WIDTH * chunk_index;
            let start_y: usize = 0;

            commands
            .spawn((SpatialBundle {
                transform: Transform::from_xyz(new_chunk_offset, chunk.y_offset, 2.),
                ..default()
            }, Chunk {
                id: chunk_index,
                x_offset: new_chunk_offset,
                y_offset: chunk.y_offset,
            }))
            .with_children(|parent| {
                for col_x in 0..CHUNK_WIDTH {
                    for col_y in 0..WORLD_HEIGHT {
                        let x = start_x + col_x;
                        let y = start_y + col_y;
                        
                        match get_block(x, y, &game_world) {
                            Block::Air => {},
                            Block::Solid(SolidBlock::Earth) => { parent.spawn((new_earth_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); },
                            Block::Solid(SolidBlock::Stone) => { parent.spawn((new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS)); },
                            Block::Solid(SolidBlock::Surface) => { parent.spawn((new_surface_block(col_x, col_y), Collider, PIXEL_PERFECT_LAYERS)); },
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
