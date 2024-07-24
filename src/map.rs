use bevy::{
    app::{Plugin, Startup, Update},
    asset::AssetServer,
    color::Color,
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::{Vec2, Vec3},
    prelude::{
        default, Commands, Component, Entity, IntoSystemConfigs, Query, Res, ResMut, Resource,
        SpatialBundle, TransformBundle,
    },
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Collider;

use crate::{
    control::MapControlOffset, game_world::GameWorld, BLOCK_SIZE, CHUNKS_TO_LOAD, CHUNK_COUNT,
    CHUNK_INITIAL_OFFSET, CHUNK_WIDTH, PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET, WORLD_CENTER_COL,
    WORLD_HEIGHT, WORLD_WIDTH,
};

const MAX_OFFSET: f32 = ((CHUNKS_TO_LOAD * CHUNK_WIDTH * BLOCK_SIZE) / 2) as f32;

#[derive(Resource)]
pub struct CurrentChunkOffset(usize);

#[derive(Component)]
struct Chunk {
    index: usize,
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
        app.add_systems(Startup, (initialize, startup).chain())
            .add_systems(Update, map_movement);
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

    for i in -half_chunks_to_load..(half_chunks_to_load + remaining_chunks_to_load) {
        let chunk_index = ((current_chunk_offset.0 as i32) + i) as usize;
        let x: f32 = (i * (CHUNK_WIDTH * BLOCK_SIZE) as i32) as f32;
        let y: f32 = (WORLD_BOTTOM_OFFSET * BLOCK_SIZE as i32) as f32;

        new_chunk(chunk_index, &game_world, x, y, &mut commands);
    }
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
            custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
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

fn new_chunk(chunk_index: usize, game_world: &GameWorld, x: f32, y: f32, commands: &mut Commands) {
    let start_x = CHUNK_WIDTH * chunk_index;
    let start_y: usize = 0;

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(x, y, 2.),
                ..default()
            },
            Chunk {
                index: chunk_index,
                x_offset: x,
                y_offset: y,
            },
        ))
        .with_children(|parent| {
            if chunk_index == (WORLD_WIDTH / CHUNK_WIDTH / 2) {
                parent.spawn((
                    new_stone_block(0, game_world.get_height(WORLD_CENTER_COL) as usize + 10),
                    PIXEL_PERFECT_LAYERS,
                ));
            }
            for col_x in 0..CHUNK_WIDTH {
                for col_y in 0..WORLD_HEIGHT {
                    let x = start_x + col_x;
                    let y = start_y + col_y;

                    match get_block(x, y, &game_world) {
                        Block::Air => {}
                        Block::Solid(SolidBlock::Earth) => {
                            parent.spawn((new_earth_block(col_x, col_y), PIXEL_PERFECT_LAYERS));
                        }
                        Block::Solid(SolidBlock::Stone) => {
                            parent.spawn((new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS));
                        }
                        Block::Solid(SolidBlock::Surface) => {
                            let v = if col_x == 0 {
                                (new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS)
                            } else {
                                (new_surface_block(col_x, col_y), PIXEL_PERFECT_LAYERS)
                            };
                            parent.spawn(v);
                        }
                    }
                }
            }
            parent.spawn((
                TransformBundle {
                    local: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..default()
                },
                Collider::polyline(new_chunk_polyline(game_world, chunk_index), Option::None),
            ));
        });
}

fn new_chunk_polyline(game_world: &GameWorld, chunk_index: usize) -> Vec<Vec2> {
    let mut vertices = Vec::<Vec2>::with_capacity(CHUNK_WIDTH * 2 + 4);

    let range_start = (chunk_index * CHUNK_WIDTH) as i32;
    vertices.push(Vec2::new(-(BLOCK_SIZE as i32 / 2) as f32, 0.0));
    vertices.push(Vec2::new(
        -(BLOCK_SIZE as i32 / 2) as f32,
        game_world.get_height(range_start as usize).trunc() * BLOCK_SIZE as f32,
    ));

    for x in range_start..(range_start + CHUNK_WIDTH as i32) {
        let y = game_world.get_height(x as usize).trunc();
        let y2 = game_world.get_height((x + 1) as usize).trunc();
        vertices.push(Vec2::new(
            ((x - range_start + 1) * (BLOCK_SIZE) as i32 - (BLOCK_SIZE / 2) as i32) as f32,
            y * BLOCK_SIZE as f32,
        ));
        if y * BLOCK_SIZE as f32 != y2 * BLOCK_SIZE as f32 {
            vertices.push(Vec2::new(
                ((x - range_start + 1) * (BLOCK_SIZE) as i32 - (BLOCK_SIZE / 2) as i32) as f32,
                y2 * BLOCK_SIZE as f32,
            ));
        }
    }
    vertices.push(Vec2::new(
        (CHUNK_WIDTH as i32 * BLOCK_SIZE as i32 - (BLOCK_SIZE / 2) as i32) as f32,
        0.0,
    ));
    vertices.push(Vec2::new(-(BLOCK_SIZE as i32 / 2) as f32, 0.0));
    vertices
}

fn map_movement(
    mut query: Query<(Entity, &mut Transform, &mut Chunk)>,
    control_offset: Res<MapControlOffset>,
    mut commands: Commands,
    game_world: Res<GameWorld>,
    mut current_chunk_offset: ResMut<CurrentChunkOffset>,
) {
    for (entity, mut transform, mut chunk) in query.iter_mut() {
        transform.translation.x -= control_offset.0;
        transform.translation.y -= control_offset.1;
        chunk.x_offset -= control_offset.0;
        chunk.y_offset -= control_offset.1;

        if (chunk.x_offset > MAX_OFFSET && control_offset.0 < 0.)
            || (chunk.x_offset < -MAX_OFFSET && control_offset.0 > 0.)
        {
            let (new_chunk_offset, next_index) = if chunk.x_offset > 0. {
                current_chunk_offset.0 -= 1;
                (
                    chunk.x_offset - MAX_OFFSET * 2.,
                    chunk.index as i32 - CHUNKS_TO_LOAD as i32,
                )
            } else {
                current_chunk_offset.0 += 1;
                (
                    chunk.x_offset + MAX_OFFSET * 2.,
                    chunk.index as i32 + CHUNKS_TO_LOAD as i32,
                )
            };
            let chunk_index = next_index as usize % CHUNK_COUNT;
            new_chunk(
                chunk_index,
                &game_world,
                new_chunk_offset,
                chunk.y_offset,
                &mut commands,
            );

            commands.entity(entity).despawn_recursive();
        }
    }
}

fn get_block(x: usize, y: usize, game_world: &GameWorld) -> Block {
    if (y as f32) < game_world.get_height(x) && (y + 1) as f32 >= game_world.get_height(x) {
        Block::Solid(SolidBlock::Surface)
    } else if (y as f32) < game_world.get_height(x) {
        Block::Solid(SolidBlock::Earth)
    } else {
        Block::Air
    }
}
