use bevy::app::{PluginGroup, PluginGroupBuilder};

use crate::{
    camera::CameraPlugin, character::CharacterPlugin, control::ControlPlugin, map::MapPlugin,
    physics::PhysicsPlugin,
};

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MapPlugin)
            .add(CameraPlugin)
            .add(ControlPlugin)
            //.add(PhysicsPlugin)
            .add_after::<MapPlugin, CharacterPlugin>(CharacterPlugin)
    }
}
