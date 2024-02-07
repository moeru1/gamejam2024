use crate::*;
use bevy::audio::PlaybackMode;

pub fn play_ost(ost: Res<OST>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: ost.0.clone(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
    });
}
