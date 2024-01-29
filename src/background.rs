use crate::constants;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use itertools::Itertools;

use crate::BackgroundImg;

#[derive(Component)]
pub struct Velocity(f32);

#[derive(Component)]
pub struct Background;

pub fn initialize_background(mut commands: Commands, texture: Handle<Image>, velocity: Velocity) {
    let sprite = |transform| SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(constants::WIDTH, constants::HEIGHT)),
            ..default()
        },
        texture: texture.clone(),
        transform,
        ..default()
    };
    commands.spawn((sprite(Transform::IDENTITY), Background));
    commands.spawn((
        sprite(Transform::from_xyz(
            constants::MAX_X + constants::HALF_WIDTH,
            0.,
            0.,
        )),
        Background,
    ));
    commands.spawn((velocity, Background));
}

pub fn background_setup(mut commands: Commands, images: Res<BackgroundImg>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: constants::WIDTH,
                height: constants::HEIGHT,
            },
            ..default()
        },
        camera: Camera {
            // renders after / on top of the main camera
            order: -1,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    initialize_background(commands, images.0.clone(), Velocity(2.0));
}

pub fn move_background(
    mut query_sprites: Query<&mut Transform, (With<Background>, Without<Velocity>)>,
    query_velocity: Query<&Velocity, With<Background>>,
    time: Res<Time>,
) {
    let (sprite1, sprite2) = query_sprites
        .iter_mut()
        .collect_tuple()
        .expect("two sprites should be here");
    let (mut transform_left, mut transform_right) = if sprite1.translation.x < sprite2.translation.x
    {
        (sprite1, sprite2)
    } else {
        (sprite2, sprite1)
    };
    assert!(transform_left.translation.x < transform_right.translation.x);

    let velocity = query_velocity.single();
    let delta_time = time.delta_seconds();
    let mut next_left_x = transform_left.translation.x - delta_time * velocity.0;
    let next_right_x = transform_right.translation.x - delta_time * velocity.0;

    if next_left_x + constants::HALF_WIDTH <= constants::MIN_X {
        let delta = constants::MIN_X - (next_left_x + constants::HALF_WIDTH);
        next_left_x = constants::WIDTH + delta;
    }

    transform_left.translation.x = next_left_x;
    transform_right.translation.x = next_right_x
}
