// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod utils;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::scene::ron::de;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::WindowResized;
use noise::core::perlin::{perlin_2d, perlin_3d};
use noise::permutationtable::PermutationTable;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{BasicMulti, NoiseFn, Perlin};
use rand::{thread_rng, Rng};

const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

const RES_WIDTH: usize = 768;
const RES_HEIGHT: usize = 432;
//const RES_WIDTH_OFFSET: usize = -(RES_WIDTH / 2);
const RES_HEIGHT_OFFSET: i32 = -((RES_HEIGHT as i32) / 2);

const BLOCK_SIZE: usize = 16;

const BLOCK_X_COUNT: usize = RES_WIDTH / BLOCK_SIZE;
const BLOCK_Y_COUNT: usize = RES_HEIGHT / BLOCK_SIZE;

const FLOOR_MEDIAN: f64 = (BLOCK_Y_COUNT as f64) * 0.5;
const FLOOR_THRESHOLD: f64 = FLOOR_MEDIAN * 0.5;

#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Canvas;

#[derive(PartialEq)]
enum Block {
    Air,
    Solid,
}

#[derive(Resource)]
struct GameWorld(NoiseMap);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, (setup, setup_block).chain())
        .add_systems(Update, fit_canvas)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, assets_server: Res<AssetServer>) {
    let m: Handle<Image> = assets_server.load("download.png");
    commands.insert_resource(GameWorld(generate_noise_map()));

    let canvas_size = Extent3d {
        width: RES_WIDTH as u32,
        height: RES_HEIGHT as u32,
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut asset_server: ResMut<AssetServer>,
    game_world: Res<GameWorld>,
) {
    let chunk: usize = 60;
    let start_x = chunk * 16;
    let start_y: usize = 0;
    
//TODO: Carregar mais chunks (o suficiente pra preencher todo o canvas + 2)
//TODO: Despawn dos chunks mais distantes
//TODO: receber algum input e usá-lo pra forçar um offset dos chunks


    //criação de um chunk
    let root = commands.spawn(
        SpatialBundle{
            transform: Transform::from_xyz(0., RES_HEIGHT_OFFSET as f32, 2.),
            ..default()
        }
    ).with_children(|parent| {
        for col_x in 0..16 {
            for col_y in 0..BLOCK_Y_COUNT {
                let val = game_world.0.get_value(col_x, col_y);
                // if val > 0.8_f64 {
                    // debug!("Value for {}:{} = {}", col_x, col_y, val);
                // }
                let x = start_x + col_x;
                let y = start_y + col_y;
                
                if get_block(x, y, &game_world.0) == Block::Solid {
                    parent.spawn(
                        (SpriteBundle {
                            sprite: Sprite {
                                color: Color::WHITE,
                                custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new((col_x * BLOCK_SIZE) as f32, (col_y * BLOCK_SIZE) as f32, 2.)),
                            ..default()
                        }, 
                        PIXEL_PERFECT_LAYERS,
                    )
                    );
                }
            }
        }
    }).id();

    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new((-4 * (BLOCK_SIZE as i32)) as f32, (4 * BLOCK_SIZE) as f32, 2.)),
        ..default()
    }, 
    PIXEL_PERFECT_LAYERS,
));
 
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

fn get_block(x: usize, y: usize, noise_map: &NoiseMap) -> Block {
    //let x_norm: f64 = (1./f64::from(BLOCK_X_COUNT) * x as f64);
    let floor = FLOOR_MEDIAN + noise_map.get_value(x as usize, 0) * FLOOR_THRESHOLD;

    if (y as f64) < floor {
        Block::Solid
    } else {
        Block::Air
    }
}

fn generate_noise_map() -> NoiseMap {
    let mut rng = thread_rng();
    //let seed: u32 = rng.gen::<_>();


    let hasher = PermutationTable::new(0);
    let r = PlaneMapBuilder::new_fn(|point| perlin_2d(point.into(), &hasher))
            .set_size(1920, 1)
            .set_x_bounds(-5., 5.)
            .set_y_bounds(-5., 5.)
            .build();
    
    utils::write_example_to_file(&r, "world.png");
    r
}
