use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText},
    EguiContexts, EguiPlugin,
};

#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct Score(pub u32);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(Score(0))
            .add_systems(Update, update_score_ui);
    }
}

fn update_score_ui(mut contexts: EguiContexts, score: Res<Score>) {
    let Score(score) = *score;
    egui::Area::new("score")
        .anchor(Align2::LEFT_TOP, (0., 25.))
        .show(contexts.ctx_mut(), |ui| {
            ui.label(
                RichText::new(format!("Score: {score}"))
                    .color(Color32::BLACK)
                    .font(FontId::proportional(72.0)),
            );
        });
}
