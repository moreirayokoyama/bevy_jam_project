use bevy::{
    app::{Plugin, PluginGroup, PluginGroupBuilder},
    prelude::Resource,
};

use crate::{
    camera::CameraPlugin, character::CharacterPlugin, control::ControlPlugin, enemy::EnemyPlugin,
    map::MapPlugin, pickables::PickablesPlugin, ui::UIPlugin,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {}
}

pub struct GamePluginGroupBuilder;

impl PluginGroup for GamePluginGroupBuilder {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(GamePlugin)
            .add(MapPlugin)
            .add(CameraPlugin)
            .add(ControlPlugin)
            .add(UIPlugin)
            .add_after::<MapPlugin, CharacterPlugin>(CharacterPlugin)
            .add_after::<GamePlugin, PickablesPlugin>(PickablesPlugin)
            .add_after::<MapPlugin, EnemyPlugin>(EnemyPlugin)
    }
}

#[derive(Resource)]
pub struct DayCount(pub i32);
