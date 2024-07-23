use bevy::{app::{FixedUpdate, Plugin}, math::{bounding::{Aabb2d, IntersectsVolume}, Vec3}, prelude::{Component, GlobalTransform, Query, Res, Transform, With, Without}, sprite::Sprite, time::Time};

#[derive(Component)]
pub struct RigidBody;

#[derive(Component)]
pub struct Collider;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(FixedUpdate, apply_gravity)
            ;
    }
}

fn apply_gravity(mut rigid_bodies: Query<(&mut Transform, &Sprite), (With<RigidBody>, Without<Collider>)>, colliders: Query<(&GlobalTransform, &Sprite), (With<Collider>, Without<RigidBody>)>, time: Res<Time>) {
    'rb_loop: for (mut rb_transform, rb_sprite) in rigid_bodies.iter_mut() {
        println!("Character: {:?}-{:?}", rb_transform.translation, rb_sprite.custom_size);
        let rb_vol: Aabb2d = Aabb2d::new( rb_transform.clone().with_translation(Vec3::new(0.0, -10.0, 0.0)).translation.truncate(), rb_sprite.custom_size.unwrap() / 2.);
        for (col_transform, col_sprite) in colliders.iter() {
            //if col_transform.compute_transform().translation.x >= -20. && col_transform.compute_transform().translation.x  <= 20. {
            //    println!("Collider: {:?}-{:?}", col_transform.compute_transform(), col_sprite.custom_size);
            //}
            let col_vol = Aabb2d::new(col_transform.compute_transform().translation.truncate(), col_sprite.custom_size.unwrap() / 2.);
            if rb_vol.intersects(&col_vol) {
                println!("Collider: {:?}-{:?}", col_transform.compute_transform(), col_sprite.custom_size);
                
                continue 'rb_loop;  
            }
        }
        rb_transform.translation.y -= 10. * time.delta_seconds();
    }
}