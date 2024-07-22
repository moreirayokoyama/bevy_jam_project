use bevy::app::{PluginGroup, PluginGroupBuilder};

use crate::{camera::CameraPlugin, control::ControlPlugin, map::MapPlugin, player::PlayerPlugin};

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
        .add(MapPlugin)
        .add(CameraPlugin)
        .add(ControlPlugin)
        .add_after::<MapPlugin, PlayerPlugin>(PlayerPlugin)
    }
}