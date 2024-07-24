use bevy::{
    app::{FixedUpdate, Plugin},
    color::palettes::css::{LIGHT_CYAN, RED},
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::{Component, Gizmos, GlobalTransform, Query, Res, Transform, With, Without},
    sprite::Sprite,
    time::Time,
};

#[derive(Component)]
pub struct RigidBody;

#[derive(Component)]
pub struct Collider;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedUpdate, apply_gravity);
    }
}

fn apply_gravity(
    mut gizmos: Gizmos,
    mut rigid_bodies: Query<(&mut Transform, &Sprite), (With<RigidBody>, Without<Collider>)>,
    colliders: Query<(&GlobalTransform, &Sprite), (With<Collider>, Without<RigidBody>)>,
    time: Res<Time>,
) {
    'rb_loop: for (mut rb_transform, rb_sprite) in rigid_bodies.iter_mut() {
        gizmos.rect_2d(
            rb_transform.clone().translation.truncate(),
            0.,
            rb_sprite.custom_size.unwrap(),
            RED,
        );

        let rb_vol: Aabb2d = Aabb2d::new(
            rb_transform.clone().translation.truncate(),
            rb_sprite.custom_size.unwrap() / 2.0,
        );
        for (col_transform, col_sprite) in colliders.iter() {
            gizmos.rect_2d(
                col_transform.clone().translation().truncate(),
                0.,
                col_sprite.custom_size.unwrap(),
                LIGHT_CYAN,
            );
            let col_vol = Aabb2d::new(
                col_transform.clone().translation().truncate(),
                col_sprite.custom_size.unwrap() / 2.0,
            );
            if rb_vol.intersects(&col_vol) {
                continue 'rb_loop;
            }
        }
        rb_transform.translation.y -= 10. * time.delta_seconds();
    }
}
