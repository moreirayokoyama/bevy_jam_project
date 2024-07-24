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
pub struct CharacterControlOffset {
    pub left: i8,
    pub right: i8,
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
    commands.insert_resource(CharacterControlOffset { left: 0, right: 0 });
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
    mut control_offset: ResMut<CharacterControlOffset>,
) {
    control_offset.left = keys.pressed(KeyCode::KeyA) as i8;
    control_offset.right = keys.pressed(KeyCode::KeyD) as i8
}
