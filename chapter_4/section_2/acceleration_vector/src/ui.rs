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

        ui.label("Horizontal Acceleration Function: a_x(x, y):");
        ui.horizontal(|ui| {
            ui.label("a_x(x, y) m/s²: ");
            let response = ui.add(egui::TextEdit::singleline(&mut settings.raw_acceleration_function_x));
            if response.changed() {
                if let Ok(expr) = settings.raw_acceleration_function_x.parse() {
                    settings.acceleration_function_x = expr;
                    settings.set_changed();
                }
            }
        });
        ui.label("Vertical Acceleration Function: a_y(x, y):");
        ui.horizontal(|ui| {
            ui.label("a_y(x, y) m/s²: ");
            let response = ui.add(egui::TextEdit::singleline(&mut settings.raw_acceleration_function_y));
            if response.changed() {
                if let Ok(expr) = settings.raw_acceleration_function_y.parse() {
                    settings.acceleration_function_y = expr;
                    settings.set_changed();
                }
            }
        });
        ui.label("Initial horizontal velocity:");
        ui.horizontal(|ui| {
            ui.label("v_x_0 m/s: ");
            let response = ui.add(egui::Slider::new(&mut settings.initial_velocity.x, -100.0..=100.0).text("m/s"));
            if response.changed() {
                settings.set_changed();
            }
        });

        ui.label("Initial vertical velocity:");
        ui.horizontal(|ui| {
            ui.label("v_y_0 m/s: ");
            let response = ui.add(egui::Slider::new(&mut settings.initial_velocity.y, -100.0..=100.0).text("m/s"));
            if response.changed() {
                settings.set_changed();
            }
        });

        ui.label("Initial position:");
        ui.horizontal(|ui| {
            ui.label("x_0 m: ");
            let response = ui.add(egui::Slider::new(&mut settings.initial_position.x, -1000.0..=1000.0).text("m"));
            if response.changed() {
                settings.set_changed();
            }
        });
        ui.horizontal(|ui| {
            ui.label("y_0 m: ");
            let response = ui.add(egui::Slider::new(&mut settings.initial_position.y, -1000.0..=1000.0).text("m"));
            if response.changed() {
                settings.set_changed();
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Launch").clicked() {
                settings.launched = true;
                settings.set_changed();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                settings.launched = false;
                settings.set_changed();
            }
        });
    });
    Ok(())
}
