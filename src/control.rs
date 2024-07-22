use bevy::{app::{Plugin, Startup, Update}, input::ButtonInput, prelude::{Commands, KeyCode, Res, ResMut, Resource}, time::Time};

use crate::MOVEMENT_SPEED;

#[derive(Resource)]
pub struct CurrentControlOffset(pub f32, pub f32);

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, startup)
            .add_systems(Update, movement_input)        
            ;
    }
}

fn startup(mut commands: Commands) {
    commands.insert_resource(CurrentControlOffset(0., 0.));
}

fn movement_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut control_offset: ResMut<CurrentControlOffset>,
    time: Res<Time>,
) {
    let xdelta = ((keys.pressed(KeyCode::ArrowRight) as i32)
        - (keys.pressed(KeyCode::ArrowLeft) as i32)) as f32 * (MOVEMENT_SPEED as f32) * time.delta_seconds();
    let ydelta = ((keys.pressed(KeyCode::ArrowUp) as i32)
        - (keys.pressed(KeyCode::ArrowDown) as i32)) as f32 * (MOVEMENT_SPEED as f32) * time.delta_seconds();

    control_offset.0 = xdelta;
    control_offset.1 = ydelta;
}
