use bevy::{app::{Plugin, Startup, Update}, asset::{Asset, AssetServer, Assets, Handle}, math::{Vec2, Vec3}, prelude::{default, Commands, Component, Image, Query, Res, With}, sprite::{Sprite, SpriteBundle}, transform::components::Transform};

use crate::{control::{CharacterControlOffset, MapControlOffset}, physics::RigidBody, GameWorld, BLOCK_SIZE, PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET_IN_PIXELS, WORLD_CENTER_COL};

#[derive(Component, Debug)]
pub struct Character;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, startup)
            .add_systems(Update, (map_following, movement))
            ;
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, game_world: Res<GameWorld>) {
    commands.spawn((
        Character, 
        RigidBody,
        SpriteBundle {
            texture: asset_server.load("character/idle/i2.png"),
            transform: Transform::from_xyz((BLOCK_SIZE as f32) * 0.5, (((game_world.surface_height[WORLD_CENTER_COL] as usize + 10) * BLOCK_SIZE) as f32) + WORLD_BOTTOM_OFFSET_IN_PIXELS as f32, 4.0),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::BottomCenter,
                custom_size: Option::Some(Vec2::new(56.0, 39.2)),
                ..default()
            },
            ..default() 
        },
        PIXEL_PERFECT_LAYERS
    ));
}

fn map_following(mut query: Query<&mut Transform, With<Character>>, control_offset: Res<MapControlOffset>) {
    let mut transform = query.single_mut();
    transform.translation.x -= control_offset.0;
    transform.translation.y -= control_offset.1;
}

fn movement(mut query: Query<&mut Transform, With<Character>>, control_offset: Res<CharacterControlOffset>, images: Res<Assets<Image>>) {
        let mut transform = query.single_mut();
        transform.translation.x += control_offset.0.trunc();
}