use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets},
    math::{UVec2, Vec2},
    prelude::{default, Commands, Component, Deref, DerefMut, Local, Query, Res, ResMut, With},
    sprite::{Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout},
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::*;

use crate::{
    control::{CharacterControlInput, MapControlOffset},
    GameWorld, BLOCK_SIZE, CHARACTER_JUMP_SPEED, CHARACTER_MOVEMENT_SPEED, CHARACTER_SIZE, GRAVITY,
    PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET_IN_PIXELS, WORLD_CENTER_COL,
};

const GROUND_TIMER: f32 = 0.5;

#[derive(Debug, Default, PartialEq)]
enum CharacterState {
    #[default]
    idle,
    walking,
    jumping,
    falling,
}

impl CharacterState {
    fn get_range(&self) -> (usize, usize) {
        match self {
            CharacterState::idle => (0, 5),
            CharacterState::walking => (8, 8),
            CharacterState::jumping => (16, 1),
            CharacterState::falling => (24, 1),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Debug)]
pub struct Character {
    movement_speed: f32,
    looking_left: bool,
    state: CharacterState,
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (map_following, movement, animate));
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
            looking_left: false,
            state: CharacterState::idle,
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
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
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
    mut query: Query<(
        &mut Character,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Sprite,
        &mut TextureAtlas,
    )>,
    control_input: Res<CharacterControlInput>,
    time: Res<Time>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
) {
    let mut next_state = CharacterState::idle;
    let delta_time = time.delta_seconds();
    let (
        mut character,
        mut character_controller,
        character_controller_output,
        mut sprite,
        mut atlas,
    ) = query.single_mut();

    let mut move_delta = Vec2::new(
        control_input.x,
        0.0, //-(character.movement_speed * BLOCK_SIZE as f32 * delta_time),
    );

    let jump_speed = control_input.y * CHARACTER_JUMP_SPEED as f32;

    if move_delta != Vec2::ZERO {
        move_delta /= move_delta.length();
    }

    if character_controller_output
        .map(|o| o.grounded)
        .unwrap_or(false)
    {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }

    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }

    move_delta.y = *vertical_movement;

    *vertical_movement += GRAVITY * delta_time * character_controller.custom_mass.unwrap_or(1.0);

    character_controller.translation =
        Some(move_delta * character.movement_speed as f32 * delta_time);

    if *vertical_movement > 0.4 {
        next_state = CharacterState::jumping;
    } else if *vertical_movement < -0.4 {
        next_state = CharacterState::falling;
    } else if move_delta.x.abs() > f32::EPSILON {
        next_state = CharacterState::walking;
    } else {
        next_state = CharacterState::idle;
    }

    if next_state != character.state {
        character.state = next_state;
        atlas.index = character.state.get_range().0;
    }

    if control_input.x != 0. {
        character.looking_left = control_input.x < 0.;
    }

    sprite.flip_x = character.looking_left;
}

fn animate(
    mut query: Query<(&Character, &mut TextureAtlas, &mut AnimationTimer)>,
    time: Res<Time>,
) {
    let (character, mut atlas, mut timer) = query.get_single_mut().unwrap();
    timer.tick(time.delta());
    if timer.just_finished() {
        atlas.index += 1;
        let (start, length) = character.state.get_range();
        if atlas.index >= start + length {
            atlas.index = start;
        }
    };
}
