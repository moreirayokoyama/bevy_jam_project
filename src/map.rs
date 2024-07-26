use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets, Handle, LoadedFolder},
    color::Color,
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    log::warn,
    math::{Rect, UVec2, Vec2, Vec3},
    prelude::*,
    render::texture::ImageSampler,
    sprite::{Sprite, SpriteBundle, TextureAtlas, TextureAtlasBuilder, TextureAtlasLayout},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Collider;

use crate::{
    control::MapControlOffset, game_world::GameWorld, BLOCK_SIZE, CHUNKS_TO_LOAD, CHUNK_COUNT,
    CHUNK_INITIAL_OFFSET, CHUNK_WIDTH, PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET, WORLD_CENTER_COL,
    WORLD_HEIGHT, WORLD_WIDTH,
};

const MAX_OFFSET: f32 = ((CHUNKS_TO_LOAD * CHUNK_WIDTH * BLOCK_SIZE) / 2) as f32;

pub struct MapPlugin;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>()
            .add_systems(OnEnter(AppState::Setup), initialize)
            .add_systems(
                Update,
                (
                    check_textures.run_if(in_state(AppState::Setup)),
                    map_movement.run_if(in_state(AppState::Finished)),
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(AppState::Finished),
                (load_textures, startup).chain(),
            );
    }
}

#[derive(Resource)]
pub struct CurrentChunkOffset(usize);

#[derive(Resource)]
struct TilesFolder(Handle<LoadedFolder>);

#[derive(Resource)]
struct Tiles {
    standard: Handle<Image>,
    white: Handle<Image>,
}

#[derive(Resource)]
struct TilesAtlasLayout(Handle<TextureAtlasLayout>);

#[derive(Component)]
struct Chunk {
    index: usize,
    x_offset: f32,
    y_offset: f32,
}

#[derive(PartialEq, Debug)]
enum Block {
    Air,
    Solid(SolidBlock),
}

#[derive(PartialEq, Debug)]
enum SolidBlock {
    Surface,
    Stone,
    Earth,
}

fn initialize(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CurrentChunkOffset(CHUNK_INITIAL_OFFSET));
    let tiles_folder = TilesFolder(asset_server.load_folder("bgp_catdev/Tillesets"));
    commands.insert_resource(tiles_folder);
}

fn load_textures(
    tiles_folder: Res<TilesFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut textures: ResMut<Assets<Image>>,
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

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    tiles_folder: Res<TilesFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&tiles_folder.0) {
            next_state.set(AppState::Finished);
        }
    }
}

fn startup(
    mut commands: Commands,
    game_world: Res<GameWorld>,
    current_chunk_offset: Res<CurrentChunkOffset>,
    atlas_layout: Res<TilesAtlasLayout>,
    tiles: Res<Tiles>,
) {
    let half_chunks_to_load = CHUNKS_TO_LOAD as i32 / 2;
    let remaining_chunks_to_load = CHUNKS_TO_LOAD as i32 % 2;

    for i in -half_chunks_to_load..(half_chunks_to_load + remaining_chunks_to_load) {
        let chunk_index = ((current_chunk_offset.0 as i32) + i) as usize;
        let x: f32 = (i * (CHUNK_WIDTH * BLOCK_SIZE) as i32) as f32;
        let y: f32 = (WORLD_BOTTOM_OFFSET * BLOCK_SIZE as i32) as f32;

        new_chunk(
            chunk_index,
            &game_world,
            x,
            y,
            &mut commands,
            &tiles,
            atlas_layout.0.clone(),
        );
    }
}

fn new_earth_block(
    x: usize,
    x_in_world: usize,
    y: usize,
    y_in_world: usize,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
    tiles: &Tiles,
    game_world: &GameWorld,
) -> (SpriteBundle, TextureAtlas) {
    let down_coord_y_in_world = if y_in_world == 0 { 0 } else { y_in_world - 1 };
    let down_coord_x_in_world = if x_in_world == 0 { 0 } else { x_in_world - 1 };

    let up_block = get_block(x_in_world, y_in_world + 1, game_world);
    let right_block = get_block(x_in_world + 1, y_in_world, game_world);
    let down_block = get_block(x_in_world, down_coord_y_in_world, game_world);
    let left_block = get_block(down_coord_x_in_world, y_in_world, game_world);

    let up_left_block = get_block(down_coord_x_in_world, y_in_world + 1, game_world);
    let up_right_block = get_block(x_in_world + 1, y_in_world + 1, game_world);
    let down_left_block = get_block(down_coord_x_in_world, down_coord_y_in_world, game_world);
    let down_right_block = get_block(x_in_world + 1, down_coord_y_in_world, game_world);

    let index = match (
        up_left_block,
        up_block,
        up_right_block,
        right_block,
        down_right_block,
        down_block,
        down_left_block,
        left_block,
    ) {
        (_, Block::Solid(_), _, Block::Solid(_), _, Block::Solid(_), _, Block::Air) => 14, // │
        (_, Block::Solid(_), _, Block::Air, _, Block::Solid(_), _, Block::Solid(_)) => 10, // │
        (
            Block::Air,
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
        ) => 12, // ┘
        (
            Block::Solid(_),
            Block::Solid(_),
            Block::Air,
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
        ) => 11, // └
        (
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Air,
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
        ) => 4, // ┌
        (
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Solid(_),
            Block::Air,
            Block::Solid(_),
        ) => 5, // ┐
        _ => 8,
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
    x_in_world: usize,
    y: usize,
    y_in_world: usize,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
    tiles: &Tiles,
    game_world: &GameWorld,
) -> (SpriteBundle, TextureAtlas) {
    let up_block = get_block(x_in_world, y_in_world + 1, game_world);
    let right_block = get_block(x_in_world + 1, y_in_world, game_world);
    let down_coord_y_in_world = if y_in_world == 0 { 0 } else { y_in_world - 1 };
    let down_block = get_block(x_in_world, down_coord_y_in_world, game_world);
    let down_coord_x_in_world = if x_in_world == 0 { 0 } else { x_in_world - 1 };
    let left_block = get_block(down_coord_x_in_world, y_in_world, game_world);

    let index = match (up_block, right_block, down_block, left_block) {
        (Block::Air, Block::Air, Block::Solid(_), Block::Solid(_)) => 3, // ┐
        (Block::Solid(_), Block::Air, Block::Air, Block::Solid(_)) => 24, // ┘
        (Block::Solid(_), Block::Solid(_), Block::Air, Block::Air) => 21, // └
        (Block::Air, Block::Solid(_), Block::Solid(_), Block::Air) => 0, // ┌
        _ => 1,
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
            transform: Transform::from_translation(Vec3::new(
                (x * BLOCK_SIZE) as f32,
                (y * BLOCK_SIZE) as f32,
                2.,
            )),
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
    y: f32,
    commands: &mut Commands,
    tiles: &Tiles,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
) {
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
                            parent.spawn((
                                new_earth_block(
                                    col_x,
                                    x,
                                    col_y,
                                    y,
                                    atlas_layout_handle.clone(),
                                    &tiles,
                                    &game_world,
                                ),
                                PIXEL_PERFECT_LAYERS,
                            ));
                        }
                        Block::Solid(SolidBlock::Stone) => {
                            parent.spawn((new_stone_block(col_x, col_y), PIXEL_PERFECT_LAYERS));
                        }
                        Block::Solid(SolidBlock::Surface) => {
                            parent.spawn((
                                new_surface_block(
                                    col_x,
                                    x,
                                    col_y,
                                    y,
                                    atlas_layout_handle.clone(),
                                    &tiles,
                                    &game_world,
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
    tiles: Res<Tiles>,
    atlas_layout_handle: Res<TilesAtlasLayout>,
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
                &tiles,
                atlas_layout_handle.0.clone(),
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

// fn create_texture_atlas(
//     folder: &LoadedFolder,
//     padding: Option<UVec2>,
//     textures: &mut ResMut<Assets<Image>>,
// ) -> (TextureAtlasLayout, Handle<Image>) {
//     // Build a texture atlas using the individual sprites
//     let mut builder = TextureAtlasBuilder::default();
//     let mut texture_atlas_builder = builder.padding(padding.unwrap_or_default());
//     for handle in folder.handles.iter() {
//         let id: bevy::prelude::AssetId<Image> = handle.id().typed_unchecked::<Image>();
//         let Some(texture) = textures.get(id) else {
//             warn!(
//                 "{:?} did not resolve to an `Image` asset.",
//                 handle.path().unwrap()
//             );
//             continue;
//         };

//         texture_atlas_builder.add_texture(Some(id), texture);
//     }

//     let (texture_atlas_layout, texture) = texture_atlas_builder.build().unwrap();
//     texture_atlas_layout.
//     let texture = textures.add(texture);

//     // Update the sampling settings of the texture atlas
//     let image = textures.get_mut(&texture).unwrap();
//     image.sampler = ImageSampler::nearest();

//     (texture_atlas_layout, texture)
// }
