use bevy::{
    app::{Plugin, Startup, Update},
    input::ButtonInput,
    prelude::{Commands, KeyCode, Res, ResMut, Resource},
    time::Time,
};

use crate::MAP_MOVEMENT_SPEED;

#[derive(Resource)]
pub struct MapControlOffset(pub f32, pub f32);

#[derive(Resource)]
pub struct CharacterControlInput {
    pub x: f32,
    pub y: f32,
}

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (map_movement_input, character_movement_input));
    }
}

fn startup(mut commands: Commands) {
    commands.insert_resource(MapControlOffset(0., 0.));
    commands.insert_resource(CharacterControlInput { x: 0., y: 0. });
}

fn map_movement_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut control_offset: ResMut<MapControlOffset>,
    time: Res<Time>,
) {
    let xdelta = (((keys.pressed(KeyCode::ArrowRight) as i32)
        - (keys.pressed(KeyCode::ArrowLeft) as i32))
        * (MAP_MOVEMENT_SPEED as i32)) as f32
        * time.delta_seconds();
    let ydelta = (((keys.pressed(KeyCode::ArrowUp) as i32)
        - (keys.pressed(KeyCode::ArrowDown) as i32))
        * (MAP_MOVEMENT_SPEED as i32)) as f32
        * time.delta_seconds();

    control_offset.0 = xdelta;
    control_offset.1 = ydelta;
}

fn character_movement_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut control_input: ResMut<CharacterControlInput>,
) {
    control_input.x =
        (-(keys.pressed(KeyCode::KeyA) as i8) + (keys.pressed(KeyCode::KeyD) as i8)) as f32;
    control_input.y = (keys.just_pressed(KeyCode::Space) as i8) as f32;
}
