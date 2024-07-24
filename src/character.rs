use bevy::{
    app::{Plugin, Startup, Update},
    asset::{Asset, AssetServer, Assets, Handle},
    color::palettes::css::{GREEN, RED},
    math::{Rect, Vec2, Vec3},
    prelude::{default, Commands, Component, DetectChanges, Gizmos, Image, Query, Res, With},
    sprite::{Sprite, SpriteBundle},
    time::Time,
    transform::components::Transform,
};
use bevy_rapier2d::prelude::*;

use crate::{
    control::{CharacterControlOffset, MapControlOffset},
    GameWorld, BLOCK_SIZE, CHARACTER_MOVEMENT_SPEED, PIXEL_PERFECT_LAYERS,
    WORLD_BOTTOM_OFFSET_IN_PIXELS, WORLD_CENTER_COL,
};

#[derive(Component, Debug)]
pub struct Character {
    movement_speed: f32,
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (map_following, movement));
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, game_world: Res<GameWorld>) {
    commands.spawn((
        Character {
            movement_speed: CHARACTER_MOVEMENT_SPEED as f32,
        },
        SpriteBundle {
            texture: asset_server.load("character/idle/i2.png"),
            transform: Transform::from_xyz(
                (BLOCK_SIZE as f32) * 0.5,
                (((game_world.surface_height[WORLD_CENTER_COL] as usize + 10) * BLOCK_SIZE) as f32)
                    + WORLD_BOTTOM_OFFSET_IN_PIXELS as f32,
                4.0,
            ),
            sprite: Sprite {
                //anchor: bevy::sprite::Anchor::BottomCenter,
                custom_size: Option::Some(Vec2::new(14.0, 30.0)),
                rect: Some(Rect {
                    max: Vec2::new(520.0, 540.0),
                    min: Vec2::new(320.0, 110.0),
                }),
                ..default()
            },
            ..default()
        },
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(11.5, 7.),
        KinematicCharacterController {
            //custom_shape:
            offset: CharacterLength::Absolute(0.1),
            autostep: Option::Some(CharacterAutostep {
                max_height: CharacterLength::Relative(0.5),
                min_width: CharacterLength::Relative(0.1),
                ..default()
            }),
            slide: true,
            snap_to_ground: Option::Some(CharacterLength::Absolute(0.1)),
            normal_nudge_factor: 4.,
            ..default()
        },
        //LockedAxes::ROTATION_LOCKED,
        PIXEL_PERFECT_LAYERS,
    ));
}

fn map_following(
    mut query: Query<&mut Transform, With<Character>>,
    control_offset: Res<MapControlOffset>,
) {
    let mut transform = query.single_mut();
    transform.translation.x -= control_offset.0;
    transform.translation.y -= control_offset.1;
}

fn movement(
    mut query: Query<(&Character, &mut KinematicCharacterController)>,
    control_offset: Res<CharacterControlOffset>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let (character, mut character_controller) = query.single_mut();

    let x_axis = -control_offset.left + control_offset.right;

    let mut move_delta = Vec2::new(x_axis as f32, -1.);
    if move_delta != Vec2::ZERO {
        move_delta /= move_delta.length();
    }

    character_controller.translation =
        Option::Some(move_delta * character.movement_speed * 4. * time.delta_seconds());
}
