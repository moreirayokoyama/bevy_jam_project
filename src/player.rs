use bevy::{app::{Plugin, Startup}, asset::AssetServer, math::Vec3, prelude::{default, Commands, Component, Res}, sprite::SpriteBundle, transform::components::Transform};

use crate::{GameWorld, BLOCK_SIZE, PIXEL_PERFECT_LAYERS, WORLD_CENTER_COL};

#[derive(Component)]
pub struct Player;


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, startup)
            ;
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, game_world: Res<GameWorld>) {
    commands.spawn((Player,
        SpriteBundle {
            texture: asset_server.load("character/idle/i2.png"),
            transform: Transform::from_xyz(-(BLOCK_SIZE as f32) / 2., game_world.surface_height[WORLD_CENTER_COL] as f32, 2.0).with_scale(Vec3::new(0.1, 0.1, 0.1)),
            ..default() 
        },
        PIXEL_PERFECT_LAYERS
    ));
}