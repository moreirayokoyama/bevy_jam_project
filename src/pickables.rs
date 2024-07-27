use bevy::hierarchy::*;
use bevy::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};
use rand::{distributions::Standard, prelude::*};

use crate::game_world;
use crate::{
    game_world::GameWorld,
    map::{Chunk, NewChunkEvent},
    BLOCK_SIZE, CHUNK_COUNT, CHUNK_WIDTH,
};

pub struct PickablesPlugin;

impl Plugin for PickablesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.observe(on_new_day)
            .observe(on_new_chunk)
            .add_systems(Startup, (initialize, load_textures, startup).chain());
    }
}

#[derive(Resource)]
struct TilesAtlasLayout(Handle<TextureAtlasLayout>);

#[derive(Resource)]
struct Tiles(Handle<Image>);

#[derive(PartialEq)]
pub enum PickableItemType {
    Diammond,
    Gem,
    Emmerald,
    Gold,
}

impl PickableItemType {
    fn get_sprite_index(&self) -> usize {
        match self {
            PickableItemType::Diammond => 1,
            PickableItemType::Gem => 2,
            PickableItemType::Emmerald => 3,
            PickableItemType::Gold => 4,
        }
    }
}

impl Distribution<PickableItemType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PickableItemType {
        match rng.gen_range(0..=3) {
            0 => PickableItemType::Diammond,
            1 => PickableItemType::Gem,
            2 => PickableItemType::Emmerald,
            _ => PickableItemType::Gold,
        }
    }
}

#[derive(Resource)]
pub struct PickableCount(u8);

#[derive(Component)]
pub struct Pickable {
    pub id: u8,
    pub item_type: PickableItemType,
    pub x: usize,
}

#[derive(Component)]
pub struct PlacedPickable {
    pub id: u8,
}

fn initialize(mut commands: Commands) {
    commands.insert_resource(PickableCount(0));
}

fn load_textures(
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 8, 5, None, None);
    commands.insert_resource(TilesAtlasLayout(texture_atlases.add(atlas_layout)));
    commands.insert_resource(Tiles(asset_server.load("purple-valley-icon-set/icons.png")));
}

fn startup(
    mut commands: Commands,
    game_world: Res<GameWorld>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    mut counter: ResMut<PickableCount>,
) {
    let pickables_count = (random::<f32>() * CHUNK_COUNT as f32) as i8;
    for i in 0..pickables_count {
        counter.0 += 1;
        commands.spawn(
            (Pickable {
                id: counter.0,
                item_type: random(),
                x: game_world.get_random_x_block(),
            }),
        );
    }
}

fn on_new_day(trigger: Trigger<NewChunkEvent>) {}

fn on_new_chunk(
    trigger: Trigger<NewChunkEvent>,
    chunks: Query<&Chunk>,
    pickables: Query<&Pickable>,
    game_world: Res<GameWorld>,
    atlas_layout: Res<TilesAtlasLayout>,
    tiles: Res<Tiles>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.chunk;
    let chunk = chunks.get(entity).unwrap();
    let chunk_x_range = (chunk.index * CHUNK_WIDTH)..((chunk.index + 1) * CHUNK_WIDTH);
    let items = pickables.iter().filter(|p| chunk_x_range.contains(&p.x));
    commands.entity(entity).with_children(|parent| {
        for i in items {
            parent.spawn((
                PlacedPickable { id: i.id },
                SpriteBundle {
                    texture: tiles.0.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        ((i.x % (CHUNK_WIDTH)) * BLOCK_SIZE) as f32,
                        game_world.get_surface(i.x) + (BLOCK_SIZE / 2) as f32,
                        2.0,
                    )),
                    ..default()
                },
                TextureAtlas {
                    layout: atlas_layout.0.clone(),
                    index: i.item_type.get_sprite_index(),
                    ..default()
                },
            ));
        }
    });
}
