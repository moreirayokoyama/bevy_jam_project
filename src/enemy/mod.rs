use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets},
    prelude::*,
    reflect::Reflect,
    sprite::TextureAtlasLayout,
    transform::components::Transform,
};
use slime::{animate_slime, slime_movement, spawn_slime};

mod slime;

use crate::{GameWorld, BLOCK_SIZE, WORLD_BOTTOM_OFFSET_IN_PIXELS, WORLD_CENTER_COL};

#[derive(Component, Reflect)]
pub struct HealthPoints {
    pub max_full_hearts: u8,
    pub current: u8,
}

impl HealthPoints {
    fn full(hearts: u8) -> Self {
        HealthPoints {
            max_full_hearts: hearts,
            current: hearts * 2,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<HealthPoints>()
            .add_systems(Startup, startup)
            .add_systems(Update, (slime_movement, animate_slime));
    }
}

fn startup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    game_world: Res<GameWorld>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    spawn_slime(
        commands,
        asset_server,
        texture_atlases,
        Transform::from_xyz(
            (BLOCK_SIZE as f32) * 0.5,
            (((game_world.get_height_in_blocks(WORLD_CENTER_COL) as usize + 10) * BLOCK_SIZE)
                as f32)
                + WORLD_BOTTOM_OFFSET_IN_PIXELS as f32,
            4.0,
        ),
    )
}
