// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod assets;
mod audio;
mod background;
mod constants;
mod hud;
mod plugin;

pub use assets::*;
use audio::*;
use background::*;
use hud::*;
use plugin::*;

use bevy::audio::PlaybackMode;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::{prelude::*, render::camera::ScalingMode};

use bevy_xpbd_3d::{math::*, prelude::*};
use rand::distributions::{Distribution, Uniform};
use rand::Rng;

#[derive(Component)]
struct Player;

#[derive(Component)]
enum Enemy {
    FrijolRojo,
    FrijolAmarillo,
    Other,
}

#[derive(Resource)]
pub struct SecondTimer(Timer);

#[derive(Resource)]
pub struct OST(pub Handle<AudioSource>);

impl SecondTimer {
    pub fn new() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Repeating))
    }
}
impl Default for SecondTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    AssetLoading,
    InGame,
    Menu,
}

#[derive(PhysicsLayer)]
enum Layer {
    Player,
    Enemy,
    Ground,
}

fn main() {
    App::new()
        .insert_resource(bevy::asset::AssetMetaCheck::Never)
        .add_state::<GameState>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // fill the entire browser window
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            CharacterControllerPlugin,
            HudPlugin,
            AssetLoaderPlugin,
            BackgroundPlugin,
        ))
        //        .add_plugins(EditorPlugin::default())
        .init_resource::<SecondTimer>()
        .add_systems(OnExit(GameState::AssetLoading), (add_background, add_ost))
        .add_systems(
            OnEnter(GameState::InGame),
            (setup, play_ost.after(background_setup)),
        )
        .add_systems(
            Update,
            (
                setup_scene_once_loaded.run_if(in_state(GameState::InGame)),
                countdown.run_if(in_state(GameState::InGame)),
                spawn_random_enemy.run_if(in_state(GameState::InGame)),
                update_score.run_if(in_state(GameState::InGame)),
                handle_collisions.run_if(in_state(GameState::InGame)),
                despawn_nonvisible_enemies.run_if(in_state(GameState::InGame)),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_gltf: Res<PlayerModel>,
) {
    // Player
    commands.spawn((
        SceneBundle {
            scene: player_gltf.0.clone(),
            transform: Transform {
                translation: Vec3::new(0., constants::MIN_Y + 1., 0.),
                rotation: Quat::from_rotation_y(PI / 3.0),
                ..default()
            },
            ..default()
        },
        CharacterControllerBundle::new(Collider::capsule(2.0, 0.5), Vector::NEG_Y * 9.81 * 2.0)
            .with_movement(30.0, 0.92, 12.0, (30.0 as Scalar).to_radians()),
        CollisionLayers::new([Layer::Player], [Layer::Enemy, Layer::Ground]),
        Player,
    ));

    //bottom
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(constants::WIDTH))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_translation(
                Vec3::NEG_Y * constants::HALF_HEIGHT - (1.0 * Vec3::Y),
            ),
            visibility: Visibility::Hidden,
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(constants::WIDTH, 0.002, 8.0),
        CollisionLayers::new([Layer::Ground], [Layer::Player]),
    ));

    let mut transform =
        Transform::from_translation(Vec3::NEG_X * constants::HALF_WIDTH + (0.5 * Vec3::X));
    transform.rotate_z(PI / 2.0);

    //left
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(constants::WIDTH))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform,
            visibility: Visibility::Hidden,
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(constants::HEIGHT, 0.002, 8.0),
        CollisionLayers::new([Layer::Ground], [Layer::Player]),
    ));

    let mut transform =
        Transform::from_translation(Vec3::X * constants::HALF_WIDTH - (0.5 * Vec3::X));
    transform.rotate_z(PI / 2.0);

    //right
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(constants::WIDTH))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform,
            visibility: Visibility::Hidden,
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(constants::HEIGHT, 0.002, 8.0),
        CollisionLayers::new([Layer::Ground], [Layer::Player]),
    ));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 6000.0,
            range: 50.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 15.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 16.,
                height: 9.,
            },
            ..default()
        }
        .into(),
        camera_3d: Camera3d {
            // don't clear the color while rendering this camera
            clear_color: ClearColorConfig::None,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 3.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

//https://bevyengine.org/examples/3D%20Rendering/3d-viewport-to-world/
fn top_screen_to_world(camera_query: Query<(&Camera, &GlobalTransform)>) -> Vec3 {
    let (camera, camera_transform) = camera_query.single();
    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, Vec2::new(0., 0.)) else {
        panic!("Could not obtain viewport_to_world");
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Z) else {
        panic!("There should be an intersection with the z plane");
    };
    let point = ray.get_point(distance);
    info!("point!!!! {}", point);
    point
}

fn countdown(time: Res<Time>, mut second_timer: ResMut<SecondTimer>) {
    second_timer.0.tick(time.delta());
}

fn spawn_random_enemy(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    enemy_scene: Res<EnemyModel>,
    second_timer: Res<SecondTimer>,
) {
    if second_timer.0.just_finished() {
        let dist = Uniform::new(constants::MIN_X, constants::MAX_X);
        let x: f32 = rand::thread_rng().sample(dist);
        let pos = Transform::from_xyz(x, 5., 0.);
        spawn_enemy(
            commands,
            meshes,
            materials,
            enemy_scene,
            Enemy::FrijolAmarillo,
            pos,
        );
    }
}

fn update_score(mut score: ResMut<Score>, second_timer: Res<SecondTimer>) {
    if second_timer.0.just_finished() {
        score.0 += 10;
    }
}

fn spawn_enemy(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    enemy_scene: Res<EnemyModel>,
    enemy: Enemy,
    transform: Transform,
) {
    match enemy {
        Enemy::FrijolAmarillo => commands.spawn((
            RigidBody::Dynamic,
            Collider::capsule(0.05, 0.05),
            CollisionLayers::new([Layer::Enemy], [Layer::Player]),
            LinearVelocity(Vec3::new(-1., 0., 0.)),
            Enemy::FrijolRojo,
            SceneBundle {
                scene: enemy_scene.amarillo.clone(),
                transform,
                ..default()
            },
        )),
        _ => unimplemented!(),
    };
}

fn despawn_nonvisible_enemies(
    mut commands: Commands,
    enemies: Query<(Entity, &ViewVisibility), With<Enemy>>,
) {
    for (entity, visibility) in &enemies {
        if !visibility.get() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_collisions(
    mut collision_event_reader: EventReader<Collision>,
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for Collision(contacts) in collision_event_reader.read() {
        let entities = [contacts.entity1, contacts.entity2];
        let enemy = entities.iter().filter_map(|&e| enemy_query.get(e).ok());
        let player = entities.iter().filter_map(|&e| player_query.get(e).ok());
        for (enemy, _) in enemy.zip(player) {
            commands.entity(enemy).despawn_recursive();
            next_state.set(GameState::Menu);
        }
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut anim_players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut anim_player in &mut anim_players {
        anim_player.play(animations.0[0].clone_weak()).repeat();
    }
}

fn add_background(mut commands: Commands, asset_background: Res<AssetBackground>) {
    commands.insert_resource(BackgroundImg(asset_background.0.clone()));
}

fn add_ost(mut commands: Commands, asset_ost: Res<AssetOST>) {
    commands.insert_resource(OST(asset_ost.0.clone()));
}
