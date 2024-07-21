use bevy::{
    app::{Plugin, Startup, Update},
    asset::Assets,
    prelude::{default, Camera2dBundle, Commands, Component, EventReader, Query, ResMut, With},
    render::{
        camera::{Camera, OrthographicProjection, RenderTarget},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::Image,
    },
    sprite::SpriteBundle,
    window::WindowResized,
};

use crate::{CANVAS_HEIGHT, CANVAS_WIDTH, HIGH_RES_LAYERS, PIXEL_PERFECT_LAYERS};

#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Canvas;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, fit_canvas);
    }
}

fn startup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: CANVAS_WIDTH as u32,
        height: CANVAS_HEIGHT as u32,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        },
        PIXEL_PERFECT_LAYERS,
    ));

    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            ..default()
        },
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2dBundle::default(), OuterCamera, HIGH_RES_LAYERS));
}

fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / CANVAS_WIDTH as f32;
        let v_scale = event.height / CANVAS_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}
