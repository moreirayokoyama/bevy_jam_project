use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets},
    math::{UVec2, Vec2},
    prelude::{default, Commands, Component, Query, Res, ResMut, With},
    sprite::{Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout},
    time::Time,
    transform::components::Transform,
};
use bevy_rapier2d::prelude::*;

use crate::{
    control::{CharacterControlOffset, MapControlOffset},
    GameWorld, BLOCK_SIZE, CHARACTER_MOVEMENT_SPEED, CHARACTER_SIZE, PIXEL_PERFECT_LAYERS,
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

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_world: Res<GameWorld>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 8, 5, None, None);
    let atlas_layout_handle = texture_atlases.add(atlas_layout);
    let texture = asset_server.load("bgp_catdev/player_and_ui/Basic_Player.png");
    commands.spawn((
        Character {
            movement_speed: CHARACTER_MOVEMENT_SPEED as f32,
        },
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(
                (BLOCK_SIZE as f32) * 0.5,
                (((game_world.get_height_in_blocks(WORLD_CENTER_COL) as usize + 10) * BLOCK_SIZE)
                    as f32)
                    + WORLD_BOTTOM_OFFSET_IN_PIXELS as f32,
                4.0,
            ),
            sprite: Sprite {
                //anchor: bevy::sprite::Anchor::BottomCenter,
                custom_size: Option::Some(Vec2::new(CHARACTER_SIZE as f32, CHARACTER_SIZE as f32)),
                ..default()
            },

            ..default()
        },
        TextureAtlas {
            layout: atlas_layout_handle,
            index: 0,
            ..Default::default()
        },
        RigidBody::KinematicPositionBased,
        Collider::capsule_y((CHARACTER_SIZE / 16) as f32, (CHARACTER_SIZE / 2) as f32),
        KinematicCharacterController {
            custom_shape: Option::Some((
                Collider::cuboid((CHARACTER_SIZE / 2) as f32, (CHARACTER_SIZE / 2) as f32),
                Vec2::new(0., CHARACTER_SIZE as f32 * 0.04),
                0.,
            )),
            offset: CharacterLength::Absolute(0.1),
            autostep: Option::Some(CharacterAutostep {
                max_height: CharacterLength::Relative(0.6),
                min_width: CharacterLength::Relative(0.1),
                ..default()
            }),
            slide: true,
            snap_to_ground: Option::Some(CharacterLength::Absolute(0.1)),
            normal_nudge_factor: 0.1,
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
) {
    let (character, mut character_controller) = query.single_mut();

    let x_axis = -control_offset.left + control_offset.right;

    let mut move_delta = Vec2::new(x_axis as f32, -1.);
    if move_delta != Vec2::ZERO {
        move_delta /= move_delta.length();
    }

    character_controller.translation = Option::Some(
        move_delta * character.movement_speed * BLOCK_SIZE as f32 * time.delta_seconds(),
    );
}
