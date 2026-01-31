use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use crate::ProjectileSettings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(EguiPlugin::default())
        .add_systems(EguiPrimaryContextPass, ui_example_system);
    }
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<ProjectileSettings>
) -> Result {
    egui::Window::new("Projectile Options").show(contexts.ctx_mut()?, |ui| {
        ui.heading("Projectile Configuration");
        
        ui.separator();

        ui.label("Radius:");
        ui.horizontal(|ui| {
        ui.label("R");
            ui.add(egui::Slider::new(&mut settings.radius, 0.0..=1000.0)
                .text("m"));
        });

        ui.label("Velocity Magnitude:");
        ui.horizontal(|ui| {
        ui.label("v");
            ui.add(egui::Slider::new(&mut settings.velocity, -1000.0..=1000.0)
                .text("m"));
        });

        ui.label("initial angle:");
        ui.horizontal(|ui| {
        ui.label("theta_0");
            ui.add(egui::Slider::new(&mut settings.initial_angle, 0.0..=2.0*std::f32::consts::PI)
                .text("rad"));
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Launch").clicked() {
                settings.launched = true;
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                settings.launched = false;
            }
        });

        ui.separator();

        // Display current values
        ui.collapsing("Current Values", |ui| {
            ui.label(format!("Centripetal Acceleration: {:.2} m/s^2", 
                settings.velocity * settings.velocity / settings.radius));
        });

    });
    Ok(())
}