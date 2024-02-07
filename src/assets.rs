use crate::GameState;
use bevy::gltf::Gltf;
use bevy::prelude::*;

#[derive(Resource)]
pub struct AssetBackground(pub Handle<Image>);

#[derive(Resource)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);

#[derive(Resource)]
pub struct PlayerModel(pub Handle<Scene>);

#[derive(Resource)]
pub struct EnemyModel {
    pub rojo: Handle<Scene>,
    pub amarillo: Handle<Scene>,
}

#[derive(Resource)]
struct AssetPackPlayer(Handle<Gltf>);

#[derive(Resource)]
struct AssetPackEnemy {
    rojo: Handle<Gltf>,
    amarillo: Handle<Gltf>,
}

#[derive(Resource)]
pub struct OST(pub Handle<AudioSource>);

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_assets).add_systems(
            Update,
            (
                load_gltf_enemy.run_if(in_state(GameState::AssetLoading)),
                load_gltf_player.run_if(in_state(GameState::AssetLoading)),
                check_if_loaded
                    .run_if(in_state(GameState::AssetLoading))
                    .after(load_gltf_player)
                    .after(load_gltf_enemy),
            ),
        );
    }
}

fn load_assets(mut commands: Commands, server: Res<AssetServer>) {
    let run: Handle<Gltf> = server.load("run.glb");
    let amarillo: Handle<Gltf> = server.load("frijol_amarillo.glb");
    let rojo: Handle<Gltf> = server.load("frijol_rojo.glb");
    let background: Handle<Image> = server.load("Background.png");
    let ost: Handle<AudioSource> = server.load("ost.flac");
    commands.insert_resource(AssetPackPlayer(run));
    commands.insert_resource(AssetPackEnemy { rojo, amarillo });
    commands.insert_resource(AssetBackground(background));
    commands.insert_resource(OST(ost));
}

fn load_gltf_player(
    mut commands: Commands,
    my: Res<AssetPackPlayer>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(gltf) = assets_gltf.get(&my.0) {
        commands.insert_resource(PlayerModel(gltf.scenes[0].clone()));
        commands.insert_resource(Animations(gltf.animations.clone()));
    }
}

fn load_gltf_enemy(
    mut commands: Commands,
    my: Res<AssetPackEnemy>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(gltf_rojo) = assets_gltf.get(&my.rojo) {
        if let Some(gltf_amarillo) = assets_gltf.get(&my.amarillo) {
            commands.insert_resource(EnemyModel {
                rojo: gltf_rojo.scenes[0].clone(),
                amarillo: gltf_amarillo.scenes[0].clone(),
            });
        }
    }
}

fn check_if_loaded(
    player: Res<AssetPackPlayer>,
    enemy: Res<AssetPackEnemy>,
    background: Res<AssetBackground>,
    ost: Res<OST>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_img: Res<Assets<Image>>,
    assets_audio: Res<Assets<AudioSource>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_loaded = assets_gltf.get(&player.0).is_some();
    let enemy_loaded =
        assets_gltf.get(&enemy.rojo).is_some() && assets_gltf.get(&enemy.amarillo).is_some();
    let image_loaded = assets_img.get(&background.0).is_some();
    let ost_loaded = assets_audio.get(&ost.0).is_some();

    if player_loaded && enemy_loaded && image_loaded && ost_loaded {
        next_state.set(GameState::InGame);
    }
}
