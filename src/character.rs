use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets},
    math::{UVec2, Vec2},
    prelude::*,
    reflect::Reflect,
    sprite::{Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout},
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::*;

use crate::{
    control::CharacterControlInput,
    pickables::{PlacedPickable, PlacedPickableCollected},
    GameWorld, BLOCK_SIZE, CHARACTER_JUMP_SPEED, CHARACTER_MOVEMENT_SPEED, CHARACTER_SIZE, GRAVITY,
    PIXEL_PERFECT_LAYERS, WORLD_BOTTOM_OFFSET_IN_PIXELS, WORLD_CENTER_COL,
};

const GROUND_TIMER: f32 = 0.5;

#[derive(Debug, Default, PartialEq)]
enum CharacterState {
    #[default]
    Idle,
    Walking,
    Jumping,
    Falling,
}

impl CharacterState {
    fn get_range(&self) -> (usize, usize) {
        match self {
            CharacterState::Idle => (0, 4),
            CharacterState::Walking => (8, 15),
            CharacterState::Jumping => (16, 16),
            CharacterState::Falling => (24, 24),
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

#[derive(Component, Reflect)]
pub struct CoinPouch(pub u64);

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

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<CoinPouch>()
            .register_type::<HealthPoints>()
            .add_systems(Startup, startup)
            .add_systems(Update, (movement, animate, handle_collision));
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
            state: CharacterState::Idle,
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
        ActiveEvents::COLLISION_EVENTS,
        KinematicCharacterController {
            custom_shape: Option::Some((
                Collider::cuboid((CHARACTER_SIZE / 3) as f32, (CHARACTER_SIZE / 2) as f32),
                Vec2::new(0., CHARACTER_SIZE as f32 * 0.04),
                0.,
            )),
            apply_impulse_to_dynamic_bodies: true,
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
        CoinPouch(50),
        HealthPoints::full(5),
        PIXEL_PERFECT_LAYERS,
    ));
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

    *vertical_movement += GRAVITY * delta_time * character_controller.custom_mass.unwrap_or(1.0);

    move_delta.y = *vertical_movement;

    let next_state = if *vertical_movement > 0.4 {
        CharacterState::Jumping
    } else if *vertical_movement < -0.4 {
        CharacterState::Falling
    } else if move_delta.x.abs() > f32::EPSILON {
        CharacterState::Walking
    } else {
        CharacterState::Idle
    };

    if next_state != character.state {
        character.state = next_state;
        atlas.index = character.state.get_range().0;
    }

    if control_input.x != 0. {
        character.looking_left = control_input.x < 0.;
    }

    sprite.flip_x = character.looking_left;

    if let Some(o) = character_controller_output {
        for c in o.collisions.iter() {
            if let Some(d) = c.hit.details {
                if (character.looking_left && d.normal1.x > 0.5)
                    || (!character.looking_left && d.normal1.x < -0.5)
                {
                    if character.state == CharacterState::Falling {
                        if control_input.y >= 0.5 {
                            move_delta.x = -move_delta.x * 30.;
                            if jump_speed > 0.0 {
                                *vertical_movement = jump_speed;
                                *grounded_timer = 0.0;
                            }
                            *vertical_movement += GRAVITY
                                * delta_time
                                * character_controller.custom_mass.unwrap_or(1.0);

                            move_delta.y = *vertical_movement;
                        } else {
                            move_delta.y /= 20.;
                        }
                    } else {
                        if control_input.y >= 0.5 {
                            move_delta.x = -move_delta.x * 30.;
                        }
                    }
                }
            }
        }
    }

    character_controller.translation =
        Some(move_delta * character.movement_speed as f32 * delta_time);
}

fn animate(
    mut query: Query<(&Character, &mut TextureAtlas, &mut AnimationTimer)>,
    time: Res<Time>,
) {
    let (character, mut atlas, mut timer) = query.get_single_mut().unwrap();
    timer.tick(time.delta());
    if timer.just_finished() {
        let (first, last) = character.state.get_range();
        atlas.index = if atlas.index == last {
            first
        } else {
            atlas.index + 1
        };
    };
}

fn handle_collision(
    mut character_controller_outputs: Query<(&KinematicCharacterControllerOutput, &mut CoinPouch)>,
    placed_pickables: Query<&PlacedPickable>,
    mut commands: Commands,
) {
    if let Ok((controller_output, mut coin_pouch)) = character_controller_outputs.get_single_mut() {
        for collision in &controller_output.collisions {
            if let Ok(placed_pickable) = placed_pickables.get(collision.entity) {
                coin_pouch.0 += placed_pickable.item_type.get_coins();
                commands.trigger(PlacedPickableCollected {
                    entity: placed_pickable.entity,
                });
                commands.entity(collision.entity).despawn();
            }
        }
    }
}
