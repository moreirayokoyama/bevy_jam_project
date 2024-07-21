use bevy::{app::{Plugin, Startup}, color::Color, hierarchy::BuildChildren, math::{Vec2, Vec3}, prelude::{default, Commands, IntoSystemConfigs, Res, Resource, SpatialBundle}, sprite::{Sprite, SpriteBundle}, transform::components::Transform};
use noise::{core::perlin::perlin_2d, permutationtable::PermutationTable, utils::{NoiseMap, PlaneMapBuilder}};

use crate::{utils, BLOCK_SIZE, BLOCK_Y_COUNT, CHUNKS_TO_LOAD, CHUNK_INITIAL_OFFSET, CHUNK_WIDTH, FLOOR_MEDIAN, FLOOR_THRESHOLD, PIXEL_PERFECT_LAYERS, RES_HEIGHT_OFFSET};

#[derive(Resource)]
struct GameWorld(NoiseMap);

#[derive(Resource)]
struct CurrentChunkOffset(usize);

#[derive(PartialEq)]
enum Block {
    Air,
    Solid,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, (initialize, startup).chain())
            ;
    }
}

fn initialize(mut commands: Commands) {
    commands.insert_resource(GameWorld(generate_noise_map()));
    commands.insert_resource(CurrentChunkOffset(CHUNK_INITIAL_OFFSET));
}

fn startup(mut commands: Commands, game_world: Res<GameWorld>, current_chunk_offset: Res<CurrentChunkOffset>) {
    for i in -(CHUNKS_TO_LOAD as i32/2)..(CHUNKS_TO_LOAD as i32/2) {
        let start_x = CHUNK_WIDTH * ((current_chunk_offset.0 as i32) + i) as usize;
        let start_y: usize = 0;

        let canvas_x: f32 = (i * (CHUNK_WIDTH * BLOCK_SIZE) as i32) as f32;
        println!("{:?}", canvas_x);

    
        //criação de um chunk
        commands.spawn(
            SpatialBundle{
                transform: Transform::from_xyz(canvas_x, RES_HEIGHT_OFFSET as f32, 2.),
                ..default()
            }
        ).with_children(|parent| {
            for col_x in 0..CHUNK_WIDTH {
                for col_y in 0..BLOCK_Y_COUNT {
                    let x = start_x + col_x;
                    let y = start_y + col_y;
                    
                    if get_block(x, y, &game_world.0) == Block::Solid {
                        parent.spawn(
                            (SpriteBundle {
                                sprite: Sprite {
                                    color: Color::WHITE,
                                    custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
                                    ..default()
                                },
                                transform: Transform::from_translation(Vec3::new((col_x * BLOCK_SIZE) as f32, (col_y * BLOCK_SIZE) as f32, 2.)),
                                ..default()
                            }, 
                            PIXEL_PERFECT_LAYERS,
                        )
                        );
                    }
                }
            }
        });
    }

    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new((-4 * (BLOCK_SIZE as i32)) as f32, (4 * BLOCK_SIZE) as f32, 2.)),
        ..default()
    }, 
    PIXEL_PERFECT_LAYERS,
));
 
}

fn get_block(x: usize, y: usize, noise_map: &NoiseMap) -> Block {
    //let x_norm: f64 = (1./f64::from(BLOCK_X_COUNT) * x as f64);
    let floor = FLOOR_MEDIAN + noise_map.get_value(x as usize, 0) * FLOOR_THRESHOLD;

    if (y as f64) < floor {
        Block::Solid
    } else {
        Block::Air
    }
}

fn generate_noise_map() -> NoiseMap {
    let hasher = PermutationTable::new(0);
    let r = PlaneMapBuilder::new_fn(|point| perlin_2d(point.into(), &hasher))
            .set_size(1920, 1)
            .set_x_bounds(-32., 32.)
            .set_y_bounds(-32., 32.)
            .build();
    
    utils::write_example_to_file(&r, "world.png");
    r
}
