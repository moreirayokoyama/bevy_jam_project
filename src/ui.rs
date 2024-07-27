use bevy::{
    app::{Plugin, Startup},
    prelude::*,
};

use crate::{
    character::{Character, CoinPouch, HealthPoints},
    HIGH_RES_LAYERS,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, (load_assets, startup).chain())
            .add_systems(FixedUpdate, (update_coins, update_health_points));
    }
}

#[derive(Component)]
struct CoinPouchNodeUI;

#[derive(Component)]
struct CoinPouchTextUI;

#[derive(Component)]
struct HealthPointsNodeUI;

#[derive(Component)]
struct HealthPointIconUI;

#[derive(Resource)]
struct TextFont(Handle<Font>);

#[derive(Resource)]
struct HeartsAndCoinsTexture(Handle<Image>);

#[derive(Resource)]
struct HeartsAndCoinsTextureAtlas(Handle<TextureAtlasLayout>);

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let font_handle: Handle<Font> = asset_server.load("fonts/courneuf-family/Courneuf-Regular.ttf");
    commands.insert_resource(TextFont(font_handle));

    let texture_handle: Handle<Image> =
        asset_server.load("bgp_catdev/player_and_ui/Basic_HeartsAndCoins.png");
    commands.insert_resource(HeartsAndCoinsTexture(texture_handle));

    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::splat(16), 5, 2, None, None);
    let texture_atlas_handle: Handle<TextureAtlasLayout> = texture_atlases.add(texture_atlas);
    commands.insert_resource(HeartsAndCoinsTextureAtlas(texture_atlas_handle));
}

fn startup(
    mut commands: Commands,
    text_font_handle: Res<TextFont>,
    texture_handle: Res<HeartsAndCoinsTexture>,
    texture_atlas_handle: Res<HeartsAndCoinsTextureAtlas>,
) {
    commands
        .spawn((
            Name::new("Main UI node"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
            HIGH_RES_LAYERS,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Health points UI"),
                HealthPointsNodeUI,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
            ));

            parent
                .spawn((
                    Name::new("Coins UI"),
                    CoinPouchNodeUI,
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        ImageBundle {
                            style: Style {
                                width: Val::Px(64.),
                                height: Val::Px(64.),
                                ..default()
                            },
                            image: UiImage::new(texture_handle.0.clone()),
                            ..default()
                        },
                        TextureAtlas {
                            layout: texture_atlas_handle.0.clone(),
                            index: 5,
                        },
                    ));

                    parent.spawn((
                        CoinPouchTextUI,
                        TextBundle::from_sections([TextSection::new(
                            "0",
                            TextStyle {
                                font_size: 42.0,
                                font: text_font_handle.0.clone(),
                                ..default()
                            },
                        )]),
                    ));
                });
        });
}

fn update_coins(
    coin_pouch_query: Query<&CoinPouch, With<Character>>,
    mut coin_pouch_style_query: Query<&mut Style, With<CoinPouchNodeUI>>,
    mut coin_pouch_text_query: Query<&mut Text, With<CoinPouchTextUI>>,
) {
    let mut coin_pouch_style = coin_pouch_style_query.single_mut();
    let mut coin_pouch_text = coin_pouch_text_query.single_mut();

    match coin_pouch_query.get_single() {
        Ok(CoinPouch(amount)) => {
            coin_pouch_text.sections[0].value = amount.to_string();
        }
        Err(_) => {
            coin_pouch_style.display = Display::None;
        }
    };
}

fn update_health_points(
    mut commands: Commands,
    health_points_query: Query<&HealthPoints, With<Character>>,
    mut health_points_ui_query: Query<(Entity, &mut Style), With<HealthPointsNodeUI>>,
    texture_handle: Res<HeartsAndCoinsTexture>,
    texture_atlas_handle: Res<HeartsAndCoinsTextureAtlas>,
) {
    let (health_points_ui_entity, mut health_points_style) = health_points_ui_query.single_mut();

    match health_points_query.get_single() {
        Ok(health_points) => {
            let mut children = vec![];

            let full_hearts = health_points.current / 2;
            let half_hearts = health_points.current % 2;

            for index in 0..health_points.max_full_hearts {
                let texture_atlas_index = if index < full_hearts {
                    0
                } else if index < full_hearts + half_hearts {
                    1
                } else {
                    2
                };

                let heart_point_entity = commands
                    .spawn((
                        HealthPointIconUI,
                        ImageBundle {
                            style: Style {
                                width: Val::Px(64.),
                                height: Val::Px(64.),
                                ..default()
                            },
                            image: UiImage::new(texture_handle.0.clone()),
                            ..default()
                        },
                        TextureAtlas {
                            layout: texture_atlas_handle.0.clone(),
                            index: texture_atlas_index,
                        },
                    ))
                    .id();
                children.push(heart_point_entity);
            }

            let mut health_points_ui_entity_commands = commands.entity(health_points_ui_entity);
            health_points_ui_entity_commands.despawn_descendants();
            health_points_ui_entity_commands.push_children(&children);
        }
        Err(_) => {
            health_points_style.display = Display::None;
        }
    };
}
