use bevy::{
    app::{Plugin, Update},
    asset::{AssetServer, Assets, Handle},
    color::Color,
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::{UVec2, Vec2, Vec3},
    prelude::*,
    sprite::{Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Collider;

use crate::{
    camera::InGameCamera, game_world::GameWorld, BLOCK_SIZE, CHUNKS_TO_LOAD, CHUNK_COUNT,
    CHUNK_INITIAL_OFFSET, CHUNK_WIDTH, PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET, WORLD_CENTER_COL,
    WORLD_HEIGHT, WORLD_WIDTH,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, (load_textures, startup).chain())
            .add_systems(Update, map_movement);
    }
}

#[derive(Resource)]
struct Tiles {
    standard: Handle<Image>,
    white: Handle<Image>,
}

#[derive(Resource)]
struct TilesAtlasLayout(Handle<TextureAtlasLayout>);

#[derive(Component)]
pub struct Chunk {
    index: usize,
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

fn load_textures(
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 7, 5, None, None);
    commands.insert_resource(TilesAtlasLayout(texture_atlases.add(atlas_layout)));
    commands.insert_resource(Tiles {
        standard: asset_server.load("bgp_catdev/Tillesets/Basic_GrassAndProps.png"),
        white: asset_server.load("bgp_catdev/Tillesets/Basic_GrassAndPropsWhiteVer.png"),
    });
}

fn startup(
    mut commands: Commands,
    game_world: Res<GameWorld>,
    atlas_layout: Res<TilesAtlasLayout>,
    tiles: Res<Tiles>,
) {
    let half_chunks_to_load = CHUNKS_TO_LOAD as i32 / 2;
    let remaining_chunks_to_load = CHUNKS_TO_LOAD as i32 % 2;

    for i in -half_chunks_to_load..(half_chunks_to_load + remaining_chunks_to_load) {
        let chunk_index = ((CHUNK_INITIAL_OFFSET as i32) + i) as usize;
        let x: f32 = (i * (CHUNK_WIDTH * BLOCK_SIZE) as i32) as f32;

        new_chunk(
            chunk_index,
            &game_world,
            x,
            &mut commands,
            &tiles,
            atlas_layout.0.clone(),
        );
    }
}

fn new_earth_block(
    x: usize,
    y: usize,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
    tiles: &Tiles,
    around_blocks: [Block; 9],
) -> (SpriteBundle, TextureAtlas) {
    let index = match around_blocks {
        [_, _, _, Block::Air, _, _, _, _, _] => 7,  // left
        [_, _, _, _, _, Block::Air, _, _, _] => 10, // right
        [Block::Air, _, _, _, _, _, _, _, _] => 12, // up left corner
        [_, _, Block::Air, _, _, _, _, _, _] => 11, // up right corner
        [_, _, _, _, _, _, _, _, Block::Air] => 4,  // down right corner
        [_, _, _, _, _, _, Block::Air, _, _] => 5,  // down left corner
        _ => 8,                                     // center
    };

    new_block_from_tilesheet(
        x,
        y,
        tiles.standard.clone(),
        atlas_layout_handle.clone(),
        index,
    )
}

fn new_stone_block(x: usize, y: usize) -> SpriteBundle {
    new_block_color(x, y, Color::linear_rgb(0.5, 0.5, 0.5))
}

fn new_surface_block(
    x: usize,
    y: usize,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
    tiles: &Tiles,
    around_blocks: [Block; 9],
) -> (SpriteBundle, TextureAtlas) {
    let index = match around_blocks {
        [_, Block::Air, _, Block::Air, _, _, _, _, _] => 0, // up left
        [_, Block::Air, _, _, _, Block::Air, _, _, _] => 3, // up right
        [_, _, _, _, _, Block::Air, _, Block::Air, _] => 24, // down right
        [_, _, _, Block::Air, _, _, Block::Air, _, _] => 21, // down left
        _ => 1,                                             // up
    };

    new_block_from_tilesheet(
        x,
        y,
        tiles.standard.clone(),
        atlas_layout_handle.clone(),
        index,
    )
}

fn new_block_from_tilesheet(
    x: usize,
    y: usize,
    tiles: Handle<Image>,
    atlas_handle: Handle<TextureAtlasLayout>,
    index: usize,
) -> (SpriteBundle, TextureAtlas) {
    (
        SpriteBundle {
            texture: tiles,
            sprite: Sprite {
                custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
                ..default()
            },
            transform: Transform::from_translation(GameWorld::get_block_position(x, y).extend(2.)),
            ..default()
        },
        TextureAtlas {
            layout: atlas_handle,
            index: index,
            ..Default::default()
        },
    )
}

fn new_block_color(x: usize, y: usize, color: Color) -> SpriteBundle {
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

fn new_chunk(
    chunk_index: usize,
    game_world: &GameWorld,
    x: f32,
    commands: &mut Commands,
    tiles: &Tiles,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
) {
    let start_x = CHUNK_WIDTH * chunk_index;
    let y = (WORLD_BOTTOM_OFFSET * BLOCK_SIZE as i32) as f32;
    let start_y: usize = 0;

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(x, y, 2.),
                ..default()
            },
            Chunk { index: chunk_index },
        ))
        .with_children(|parent| {
            if chunk_index == (WORLD_WIDTH / CHUNK_WIDTH / 2) {
                parent.spawn((
                    new_stone_block(
                        0,
                        game_world.get_height_in_blocks(WORLD_CENTER_COL) as usize + 10,
                    ),
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
                            let around_blocks = get_around_blocks(x, y, game_world);

                            parent.spawn((
                                new_earth_block(
                                    col_x,
                                    col_y,
                                    atlas_layout_handle.clone(),
                                    &tiles,
                                    around_blocks,
                                ),
                                PIXEL_PERFECT_LAYERS,
                            ));
                        }
                        Block::Solid(SolidBlock::Stone) => {
                            parent.spawn((new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS));
                        }
                        Block::Solid(SolidBlock::Surface) => {
                            let around_blocks = get_around_blocks(x, y, game_world);

                            parent.spawn((
                                new_surface_block(
                                    col_x,
                                    col_y,
                                    atlas_layout_handle.clone(),
                                    &tiles,
                                    around_blocks,
                                ),
                                PIXEL_PERFECT_LAYERS,
                            ));
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
        game_world.get_surface(range_start as usize),
    ));

    for x in range_start..(range_start + CHUNK_WIDTH as i32) {
        let y = game_world.get_surface(x as usize);
        let y2 = game_world.get_surface((x + 1) as usize);
        vertices.push(Vec2::new(
            ((x - range_start + 1) * (BLOCK_SIZE) as i32 - (BLOCK_SIZE / 2) as i32) as f32,
            y,
        ));
        if y * BLOCK_SIZE as f32 != y2 {
            vertices.push(Vec2::new(
                ((x - range_start + 1) * (BLOCK_SIZE) as i32 - (BLOCK_SIZE / 2) as i32) as f32,
                y2,
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
    cam_query: Query<&InGameCamera>,
    query: Query<(Entity, &Transform, &Chunk)>,
    mut commands: Commands,
    game_world: Res<GameWorld>,
    tiles: Res<Tiles>,
    atlas_layout_handle: Res<TilesAtlasLayout>,
) {
    let camera = cam_query.single();
    for (entity, transform, chunk) in query.iter() {
        let camera_offset = camera.translation.x - transform.translation.x;
        if (camera.is_going_right
            && camera_offset > camera.chunk_unload_after
            && transform.translation.x < camera.translation.x)
            || (!camera.is_going_right
                && camera_offset < -camera.chunk_unload_after
                && transform.translation.x > camera.translation.x)
        {
            let (new_chunk_offset, next_index) = if camera.is_going_right {
                (
                    transform.translation.x + (CHUNKS_TO_LOAD * CHUNK_WIDTH * BLOCK_SIZE) as f32,
                    chunk.index as i32 + CHUNKS_TO_LOAD as i32,
                )
            } else {
                (
                    transform.translation.x - (CHUNKS_TO_LOAD * CHUNK_WIDTH * BLOCK_SIZE) as f32,
                    chunk.index as i32 - CHUNKS_TO_LOAD as i32,
                )
            };
            let chunk_index = ((next_index + CHUNK_COUNT as i32) % CHUNK_COUNT as i32) as usize;
            new_chunk(
                chunk_index,
                &game_world,
                new_chunk_offset,
                &mut commands,
                &tiles,
                atlas_layout_handle.0.clone(),
            );

            commands.entity(entity).despawn_recursive();
        }
    }
}

fn get_block(x: usize, y: usize, game_world: &GameWorld) -> Block {
    if (y as f32) < game_world.get_height_in_blocks(x)
        && (y + 1) as f32 >= game_world.get_height_in_blocks(x)
    {
        Block::Solid(SolidBlock::Surface)
    } else if (y as f32) < game_world.get_height_in_blocks(x) {
        Block::Solid(SolidBlock::Earth)
    } else {
        Block::Air
    }
}

fn get_around_blocks(x: usize, y: usize, game_world: &GameWorld) -> [Block; 9] {
    let up_x = x + 1;
    let down_x = if x == 0 { 0 } else { x - 1 };
    let up_y = y + 1;
    let down_y = if y == 0 { 0 } else { y - 1 };

    let up_left_block = get_block(down_x, up_y, game_world);
    let up_block = get_block(x, up_y, game_world);
    let up_right_block = get_block(up_x, up_y, game_world);
    let left_block = get_block(down_x, y, game_world);
    let center_block = get_block(x, y, game_world);
    let right_block = get_block(up_x, y, game_world);
    let down_left_block = get_block(down_x, down_y, game_world);
    let down_block = get_block(x, down_y, game_world);
    let down_right_block = get_block(up_x, down_y, game_world);

    [
        up_left_block,
        up_block,
        up_right_block,
        left_block,
        center_block,
        right_block,
        down_left_block,
        down_block,
        down_right_block,
    ]
}
