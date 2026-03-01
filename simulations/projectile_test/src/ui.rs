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
        
        // Initial velocity components
        ui.label("Initial Velocity:");
        ui.horizontal(|ui| {
            ui.label("X: ");
            ui.add(egui::Slider::new(&mut settings.initial_velocity.0.x, -100.0..=100.0)
                .text("m/s"));
        });
        ui.horizontal(|ui| {
            ui.label("Y: ");
            ui.add(egui::Slider::new(&mut settings.initial_velocity.0.y, -100.0..=100.0)
                .text("m/s"));
        });
        
        ui.separator();
        
        // Gravitational constant
        ui.horizontal(|ui| {
            ui.label("Gravity: ");
            ui.add(egui::Slider::new(&mut settings.gravitational_constant, -300.0..=0.0)
                .text("m/s²"));
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

        // Display current values
        ui.collapsing("Current Values", |ui| {
            ui.label(format!("Velocity: ({:.2}, {:.2}) m/s", 
                settings.initial_velocity.0.x, 
                settings.initial_velocity.0.y));
            ui.label(format!("Gravity: {:.2} m/s²", settings.gravitational_constant));
        });
    });
    Ok(())
}