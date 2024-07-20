// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::WindowResized;
use noise::{NoiseFn, Perlin, Seedable};

const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

const RES_WIDTH: u32 = 768;
const RES_HEIGHT: u32 = 432;

#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Canvas;

#[derive(PartialEq)]
enum Block {
    Air,
    Solid,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, (setup, setup_block))
        .add_systems(Update, fit_canvas)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
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

fn setup_block(mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,) {

        for i in 0..RES_WIDTH / 4 {
            for j in 0..RES_HEIGHT / 4 {
                if get_block(i, j) == Block::Solid {

                    commands.spawn((
                        MaterialMesh2dBundle {
                            mesh: meshes.add(Rectangle::default()).into(),
                            transform: Transform::from_xyz(((i * 4) as f32) - ((RES_WIDTH / 2) as f32), ((j * 4) as f32) - ((RES_HEIGHT / 2) as f32), 2.).with_scale(Vec3::splat(16.)),
                            material: materials.add(Color::WHITE),
                            ..default()
                        },
                        //Rotate,
                        PIXEL_PERFECT_LAYERS,
                    ));
                }
            
            }
        }

}

fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}

fn get_block(x: u32, y: u32) -> Block {
    let x_norm: f64 = 1./192. * x as f64;
    let perlin = Perlin::new(2);
    let val = 50. + perlin.get([x_norm, 0.]) * 20.;

    if (y as f64) < val {
        Block::Solid
    } else {
        Block::Air
    }
}
