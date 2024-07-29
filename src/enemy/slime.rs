use bevy::{
    asset::{AssetServer, Assets},
    math::{UVec2, Vec2},
    prelude::*,
    sprite::{Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout},
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::*;

use crate::{BLOCK_SIZE, GRAVITY, PIXEL_PERFECT_LAYERS};

use super::HealthPoints;

const SLIME_SIZE: f32 = (BLOCK_SIZE * 2) as f32;

#[derive(Debug, Default, PartialEq)]
enum SlimeState {
    #[default]
    Idle,
    Walking,
    Falling,
}

impl SlimeState {
    fn get_range(&self) -> (usize, usize) {
        match self {
            SlimeState::Idle => (0, 1),
            SlimeState::Walking => (0, 5),
            SlimeState::Falling => (2, 2),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component, Debug)]
pub struct Slime {
    movement_speed: f32,
    looking_left: bool,
    state: SlimeState,
}

#[derive(Component)]
pub struct Direction {
    x: f32,
}

pub fn spawn_slime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    position: Transform,
) {
    let atlas_layout =
        TextureAtlasLayout::from_grid(UVec2::new(16, 16), 6, 1, None, Some(UVec2::new(16, 16)));
    let atlas_layout_handle = texture_atlases.add(atlas_layout);
    let texture = asset_server.load("enemies/red-slime-spritesheet.png");

    commands.spawn((
        Slime {
            movement_speed: 80.0,
            looking_left: false,
            state: SlimeState::Idle,
        },
        SpriteBundle {
            texture,
            transform: position,
            sprite: Sprite {
                custom_size: Option::Some(Vec2::new(SLIME_SIZE, SLIME_SIZE)),
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
        Collider::capsule_y(SLIME_SIZE / 16.0, SLIME_SIZE / 2.0),
        KinematicCharacterController {
            custom_shape: Option::Some((
                Collider::cuboid(SLIME_SIZE / 3.0, SLIME_SIZE / 2.0),
                Vec2::new(0., SLIME_SIZE * 0.04),
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
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        HealthPoints::full(1),
        Direction { x: 1.0 },
        PIXEL_PERFECT_LAYERS,
    ));
}

pub fn slime_movement(
    mut query: Query<(
        &mut Slime,
        &mut Direction,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Sprite,
        &mut TextureAtlas,
    )>,
    time: Res<Time>,
    mut vertical_movement: Local<f32>,
) {
    let delta_time = time.delta_seconds();
    let (
        mut enemy,
        mut direction,
        mut enemy_controller,
        enemy_controller_output,
        mut sprite,
        mut atlas,
    ) = query.single_mut();

    let x = direction.x;

    let mut move_delta = Vec2::new(x, 0.0);

    if move_delta != Vec2::ZERO {
        move_delta /= move_delta.length();
    }

    if enemy_controller_output.map(|o| o.grounded).unwrap_or(false) {
        *vertical_movement = 0.0;
    }

    *vertical_movement += GRAVITY * delta_time * enemy_controller.custom_mass.unwrap_or(1.0);

    move_delta.y = *vertical_movement;

    let next_state = if *vertical_movement < -0.4 {
        SlimeState::Falling
    } else if move_delta.x.abs() > f32::EPSILON {
        SlimeState::Walking
    } else {
        SlimeState::Idle
    };

    if next_state != enemy.state {
        enemy.state = next_state;
        atlas.index = enemy.state.get_range().0;
    }

    if x != 0. {
        enemy.looking_left = x < 0.;
    }

    sprite.flip_x = enemy.looking_left;

    if let Some(o) = enemy_controller_output {
        for c in o.collisions.iter() {
            if let Some(d) = c.hit.details {
                if (enemy.looking_left && d.normal1.x > 0.5)
                    || (!enemy.looking_left && d.normal1.x < -0.5)
                {
                    move_delta.y /= 20.;
                }

                if enemy.looking_left && d.normal1.x > 0.5 {
                    direction.x = 1.0;
                }

                if !enemy.looking_left && d.normal1.x < -0.5 {
                    direction.x = -1.0;
                }
            }
        }
    }

    enemy_controller.translation = Some(move_delta * enemy.movement_speed as f32 * delta_time);
}

pub fn animate_slime(
    mut query: Query<(&Slime, &mut TextureAtlas, &mut AnimationTimer)>,
    time: Res<Time>,
) {
    for (enemy, mut atlas, mut timer) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            let (first, last) = enemy.state.get_range();
            atlas.index = if atlas.index == last {
                first
            } else {
                atlas.index + 1
            };
        };
    }
}
