use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use crate::ProjectileSettings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(EguiPlugin::default())
        .add_systems(EguiPrimaryContextPass, ui_system);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<ProjectileSettings>
) -> Result {
    egui::Window::new("Projectile Options").show(contexts.ctx_mut()?, |ui| {
        ui.heading("Projectile Configuration");

        ui.separator();

        ui.label("Horizontal Acceleration Function: a(x, y):");
        ui.horizontal(|ui| {
            ui.label("a_x(x, y) m/s²: ");
            let response = ui.add(egui::TextEdit::singleline(&mut settings.raw_acceleration_function_x));
            if response.lost_focus() {
                if let Ok(expr) = settings.raw_acceleration_function_x.parse() {
                    settings.acceleration_function_x = expr;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("a_y(x, y) m/s²: ");
            let response = ui.add(egui::TextEdit::singleline(&mut settings.raw_acceleration_function_y));
            if response.lost_focus() {
                if let Ok(expr) = settings.raw_acceleration_function_y.parse() {
                    settings.acceleration_function_y = expr;
                }
            }
        });

        ui.separator();

        // launch button
        ui.horizontal(|ui| {
            if ui.button("Launch").clicked() {
                settings.launched = true;
            }
        });

        // reset button
        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                settings.launched = false;
            }
        });
    });
    Ok(())
}
