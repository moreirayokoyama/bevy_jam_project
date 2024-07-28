use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    input::{mouse::MouseWheel, ButtonInput},
    math::Vec3,
    prelude::{
        default, Camera2dBundle, Commands, Component, EventReader, KeyCode, Query, Res, ResMut,
        Transform, With, Without,
    },
    render::{
        camera::{Camera, OrthographicProjection, RenderTarget},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::Image,
        view::Msaa,
    },
    sprite::SpriteBundle,
    window::WindowResized,
};

use crate::{
    character::Character, map::Chunk, BACKGROUND_LAYERS, BLOCK_SIZE, CAMERA_REGULAR_SPEED,
    CANVAS_HEIGHT, CANVAS_WIDTH, CHARACTER_MOVEMENT_SPEED, CHARACTER_ROAMING_THRESHOLD,
    CHUNKS_TO_LOAD, CHUNK_WIDTH, HIGH_RES_LAYERS, PIXEL_PERFECT_LAYERS,
};

#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Canvas;

#[derive(Component)]
struct Background;

#[derive(Component)]
pub struct InGameCamera {
    pub is_going_right: bool,
    pub translation: Vec3,
    pub chunk_unload_after: f32,
    pub state: CameraState,
    pub char_roaming_threshold: f32,
    pub catching_up: f32,
    pub speed: f32,
    pub zoom_step: f32,
    pub zoom_min_max: (f32, f32),
}

#[derive(PartialEq, Default)]
pub enum CameraState {
    #[default]
    Waiting,
    Moving,
    CatchingUp,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Msaa::Off)
            .add_systems(Startup, startup)
            .add_systems(Update, (fit_canvas, move_camera));
    }
}

fn startup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
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

    let bg_canvas = canvas.clone();

    let image_handle = images.add(canvas);
    let bg_handle = images.add(bg_canvas);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -2,
                target: RenderTarget::Image(bg_handle),
                ..default()
            },
            ..default()
        },
        BACKGROUND_LAYERS,
    ));

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
        InGameCamera {
            is_going_right: false,
            translation: Vec3::ZERO,
            chunk_unload_after: (((CHUNKS_TO_LOAD / 2) * CHUNK_WIDTH * BLOCK_SIZE) as f32),
            state: CameraState::Waiting,
            char_roaming_threshold: CHARACTER_ROAMING_THRESHOLD as f32,
            catching_up: 0.,
            speed: 0.,
            zoom_step: -0.1,
            zoom_min_max: (0.4, 1.5),
        },
        PIXEL_PERFECT_LAYERS,
    ));

    let bg_image: Handle<Image> = asset_server.load("bgp_catdev/BackGrounds/Basic_BackGround.png");

    commands.spawn((
        SpriteBundle {
            texture: bg_image,
            transform: Transform::from_scale(Vec3::new(6., 5.25, 0.0)),
            ..default()
        },
        Canvas,
        Background,
        BACKGROUND_LAYERS,
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

fn move_camera(
    mut cam_query: Query<
        (
            &mut Transform,
            &mut OrthographicProjection,
            &mut InGameCamera,
        ),
        (Without<Character>, Without<Chunk>),
    >,
    keys: Res<ButtonInput<KeyCode>>,
    char_query: Query<&Transform, (With<Character>, Without<InGameCamera>)>,
    mut bg_query: Query<
        &mut Transform,
        (With<Background>, Without<InGameCamera>, Without<Character>),
    >,
    mut evr_scroll: EventReader<MouseWheel>,
) {
    let (mut transform, mut projection, mut camera) = cam_query.single_mut();

    let char = char_query.single();
    let mut bg = bg_query.single_mut();

    if camera.state == CameraState::Waiting {
        let char_offset = char.translation.x.abs();
        if char_offset > camera.char_roaming_threshold {
            camera.is_going_right = char.translation.x > 0.;
            camera.state = CameraState::CatchingUp;
            camera.catching_up = char_offset;
        }
        return;
    } else if camera.state == CameraState::CatchingUp {
        camera.speed = 10.
            * CAMERA_REGULAR_SPEED as f32
            * (projection.scale / ((CHARACTER_MOVEMENT_SPEED as f32) * 2.));
        camera.catching_up -= camera.speed.abs();
        if camera.catching_up <= 0. {
            camera.catching_up = 0.;
            camera.state = CameraState::Moving;
        }
    } else {
        for ev in evr_scroll.read() {
            projection.scale += ev.y * camera.zoom_step;
            projection.scale = projection
                .scale
                .clamp(camera.zoom_min_max.0, camera.zoom_min_max.1);
        }

        if keys.pressed(KeyCode::ShiftLeft) {
            camera.speed = (CAMERA_REGULAR_SPEED as f32)
                * 5.
                * (projection.scale / ((CHARACTER_MOVEMENT_SPEED as f32) * 2.));
        } else {
            camera.speed = CAMERA_REGULAR_SPEED as f32
                * (projection.scale / ((CHARACTER_MOVEMENT_SPEED as f32) * 2.));
        }
    }

    let direction = if camera.is_going_right { 1. } else { -1. };
    let x_offset = direction * camera.speed;
    transform.translation.x += x_offset;
    transform.translation.y = char.translation.y;
    camera.translation = transform.translation.clone();

    bg.translation = transform.translation.clone();
}
