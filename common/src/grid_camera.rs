//! Grid Camera Plugin
//! 
//! Provides a 2D camera with zoom, pan, and a dynamic grid coordinate system.
//! Features:
//! - Mouse wheel zoom (with orthographic projection scaling)
//! - Click-and-drag panning (egui-aware)
//! - Dynamic grid that adapts to zoom level
//! - Axis-anchored labels
//! - Interactive tooltips showing coordinates at grid intersections

use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseMotion};

#[cfg(feature = "egui")]
use bevy_egui::EguiContexts;

/// Plugin that adds camera controls and grid visualization
pub struct GridCameraPlugin {
    /// Minimum zoom level (higher = more zoomed out)
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
    /// Maximum distance camera can pan from origin
    pub max_pan_distance: f32,
}

impl Default for GridCameraPlugin {
    fn default() -> Self {
        Self {
            min_zoom: 0.01,
            max_zoom: 5.0,
            max_pan_distance: 2000.0,
        }
    }
}

impl Plugin for GridCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraZoom {
            zoom: 1.0,
            min_zoom: self.min_zoom,
            max_zoom: self.max_zoom,
            last_zoom: -1.0, // Set to -1.0 to trigger initial grid generation
            last_camera_pos: Vec2::ZERO,
        })
        .insert_resource(CameraDragState::default())
        .insert_resource(CameraBounds {
            max_distance: self.max_pan_distance,
        })
        .add_systems(Startup, setup_grid_camera)
        .add_systems(Update, (
            handle_camera_zoom,
            handle_camera_pan,
            update_dynamic_grid,
            update_grid_tooltip,
        ));
    }
}

#[derive(Component)]
struct GridLine;

#[derive(Component)]
struct GridLabel;

#[derive(Component)]
struct GridTooltip;

#[derive(Resource)]
struct CameraZoom {
    zoom: f32,
    min_zoom: f32,
    max_zoom: f32,
    last_zoom: f32,
    last_camera_pos: Vec2,
}

#[derive(Resource, Default)]
struct CameraDragState {
    is_dragging: bool,
}

#[derive(Resource)]
struct CameraBounds {
    max_distance: f32,
}

fn setup_grid_camera(mut commands: Commands) {
    // Spawn camera with projection for zooming
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn handle_camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    mut zoom: ResMut<CameraZoom>,
) {
    for event in scroll_events.read() {
        let zoom_delta = -event.y * 0.1;
        let new_zoom = (zoom.zoom + zoom_delta).clamp(zoom.min_zoom, zoom.max_zoom);
        
        zoom.zoom = new_zoom;
        
        if let Ok(mut projection) = camera_query.single_mut() {
            if let Projection::Orthographic(ref mut ortho) = projection.as_mut() {
                ortho.scale = zoom.zoom;
            }
        }
    }
}

#[cfg(feature = "egui")]
fn handle_camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: MessageReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut drag_state: ResMut<CameraDragState>,
    zoom: Res<CameraZoom>,
    bounds: Res<CameraBounds>,
    mut contexts: EguiContexts,
) {
    // Check if egui wants the mouse (hovering over UI or interacting with it)
    let egui_wants_mouse = if let Ok(ctx) = contexts.ctx_mut() {
        ctx.is_pointer_over_area() || ctx.wants_pointer_input()
    } else {
        false
    };
    
    if mouse_button.just_pressed(MouseButton::Left) && !egui_wants_mouse {
        drag_state.is_dragging = true;
    }
    
    if mouse_button.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
    }
    
    // Also stop dragging if egui starts wanting the mouse mid-drag
    if egui_wants_mouse && drag_state.is_dragging {
        drag_state.is_dragging = false;
    }
    
    if drag_state.is_dragging {
        let Ok(mut camera_transform) = camera_query.single_mut() else {
            return;
        };
        
        for event in motion_events.read() {
            let pan_speed = zoom.zoom;
            camera_transform.translation.x -= event.delta.x * pan_speed;
            camera_transform.translation.y += event.delta.y * pan_speed;
            
            camera_transform.translation.x = camera_transform.translation.x
                .clamp(-bounds.max_distance, bounds.max_distance);
            camera_transform.translation.y = camera_transform.translation.y
                .clamp(-bounds.max_distance, bounds.max_distance);
        }
    }
}

#[cfg(not(feature = "egui"))]
fn handle_camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: MessageReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut drag_state: ResMut<CameraDragState>,
    zoom: Res<CameraZoom>,
    bounds: Res<CameraBounds>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        drag_state.is_dragging = true;
    }
    
    if mouse_button.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
    }
    
    if drag_state.is_dragging {
        let Ok(mut camera_transform) = camera_query.single_mut() else {
            return;
        };
        
        for event in motion_events.read() {
            let pan_speed = zoom.zoom;
            camera_transform.translation.x -= event.delta.x * pan_speed;
            camera_transform.translation.y += event.delta.y * pan_speed;
            
            camera_transform.translation.x = camera_transform.translation.x
                .clamp(-bounds.max_distance, bounds.max_distance);
            camera_transform.translation.y = camera_transform.translation.y
                .clamp(-bounds.max_distance, bounds.max_distance);
        }
    }
}

fn calculate_grid_spacing(zoom: f32) -> f32 {
    let exponent = (zoom * 2.0).log2();
    let spacing = 50.0 * 2_f32.powf(exponent.floor());
    spacing.max(1.0)
}

fn update_dynamic_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut zoom: ResMut<CameraZoom>,
    camera_query: Query<(&Transform, &Projection), With<Camera2d>>,
    grid_query: Query<Entity, With<GridLine>>,
    label_query: Query<Entity, With<GridLabel>>,
    windows: Query<&Window>,
) {
    let Ok((camera_transform, _projection)) = camera_query.single() else {
        return;
    };
    
    let camera_pos = camera_transform.translation.truncate();
    
    zoom.last_zoom = zoom.zoom;
    zoom.last_camera_pos = camera_pos;
    
    for entity in grid_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in label_query.iter() {
        commands.entity(entity).despawn();
    }
    
    let Ok(window) = windows.single() else {
        return;
    };
    
    let window_size = Vec2::new(window.width(), window.height());
    let visible_half_size = window_size * zoom.zoom / 2.0;
    let min_visible = camera_pos - visible_half_size;
    let max_visible = camera_pos + visible_half_size;
    
    let grid_spacing = calculate_grid_spacing(zoom.zoom);
    let label_frequency = 1;
    let label_size = 14.0;
    
    let padding = 2;
    let min_i = ((min_visible.x / grid_spacing).floor() as i32) - padding;
    let max_i = ((max_visible.x / grid_spacing).ceil() as i32) + padding;
    let min_j = ((min_visible.y / grid_spacing).floor() as i32) - padding;
    let max_j = ((max_visible.y / grid_spacing).ceil() as i32) + padding;
    
    let grid_color = Color::srgba(0.3, 0.3, 0.3, 0.5);
    let axis_color = Color::srgba(0.5, 0.5, 0.5, 0.8);
    let label_color = Color::srgba(0.7, 0.7, 0.7, 0.9);
    
    let line_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let grid_material = materials.add(grid_color);
    let axis_material = materials.add(axis_color);
    
    let x_axis_visible = min_visible.y <= 0.0 && max_visible.y >= 0.0;
    let y_axis_visible = min_visible.x <= 0.0 && max_visible.x >= 0.0;
    
    // Vertical lines
    for i in min_i..=max_i {
        let x = i as f32 * grid_spacing;
        let is_axis = i == 0;
        let material = if is_axis { axis_material.clone() } else { grid_material.clone() };
        // Scale line width with zoom to maintain constant screen width
        let base_width = if is_axis { 2.0 } else { 1.0 };
        let width = base_width * zoom.zoom;
        let z_depth = if is_axis { -0.5 } else { -1.0 }; // Axis lines in front
        
        commands.spawn((
            Mesh2d(line_mesh.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(x, camera_pos.y, z_depth)
                .with_scale(Vec3::new(width, visible_half_size.y * 3.0, 1.0)),
            GridLine,
        ));
        
        if x_axis_visible && i != 0 && i % label_frequency == 0 {
            commands.spawn((
                Text2d::new(format!("{}", x as i32)),
                TextFont {
                    font_size: label_size,
                    ..default()
                },
                TextColor(label_color),
                Transform::from_xyz(x, -20.0 * zoom.zoom, 0.0)
                    .with_scale(Vec3::splat(zoom.zoom)), // Scale text with zoom to maintain screen size
                GridLabel,
            ));
        }
    }
    
    // Horizontal lines
    for j in min_j..=max_j {
        let y = j as f32 * grid_spacing;
        let is_axis = j == 0;
        let material = if is_axis { axis_material.clone() } else { grid_material.clone() };
        // Scale line width with zoom to maintain constant screen width
        let base_width = if is_axis { 2.0 } else { 1.0 };
        let width = base_width * zoom.zoom;
        let z_depth = if is_axis { -0.5 } else { -1.0 }; // Axis lines in front
        
        commands.spawn((
            Mesh2d(line_mesh.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(camera_pos.x, y, z_depth)
                .with_scale(Vec3::new(visible_half_size.x * 3.0, width, 1.0)),
            GridLine,
        ));
        
        if y_axis_visible && j != 0 && j % label_frequency == 0 {
            commands.spawn((
                Text2d::new(format!("{}", y as i32)),
                TextFont {
                    font_size: label_size,
                    ..default()
                },
                TextColor(label_color),
                Transform::from_xyz(-30.0 * zoom.zoom, y, 0.0)
                    .with_scale(Vec3::splat(zoom.zoom)), // Scale text with zoom to maintain screen size
                GridLabel,
            ));
        }
    }
    
    // Origin label
    if x_axis_visible && y_axis_visible {
        commands.spawn((
            Text2d::new("(0, 0)"),
            TextFont {
                font_size: label_size * 0.7,
                ..default()
            },
            TextColor(label_color),
            Transform::from_xyz(-10.0 * zoom.zoom, -10.0 * zoom.zoom, 0.0)
                .with_scale(Vec3::splat(zoom.zoom)), // Scale text with zoom to maintain screen size
            GridLabel,
        ));
    }
}

fn update_grid_tooltip(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    tooltip_query: Query<Entity, With<GridTooltip>>,
    zoom: Res<CameraZoom>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    
    for entity in tooltip_query.iter() {
        commands.entity(entity).despawn();
    }
    
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    
    // Check if near a grid intersection
    let grid_spacing = calculate_grid_spacing(zoom.zoom);
    let nearest_x = (world_position.x / grid_spacing).round() * grid_spacing;
    let nearest_y = (world_position.y / grid_spacing).round() * grid_spacing;
    let nearest_point = Vec2::new(nearest_x, nearest_y);
    
    let hover_threshold = 15.0 * zoom.zoom;
    let is_near_intersection = world_position.distance(nearest_point) < hover_threshold;
    
    // Show tooltip - snap to grid intersection if near one, otherwise show cursor position
    let (tooltip_pos, tooltip_text) = if is_near_intersection {
        (nearest_point, format!("({:.0}, {:.0})", nearest_x, nearest_y))
    } else {
        (world_position, format!("({:.0}, {:.0})", world_position.x, world_position.y))
    };
    
    let label_size = 14.0;
    let offset = 20.0 * zoom.zoom;
    
    commands.spawn((
        Text2d::new(tooltip_text),
        TextFont {
            font_size: label_size * 1.2,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.6, 0.95)),
        Transform::from_xyz(tooltip_pos.x + offset, tooltip_pos.y + offset, 10.0)
            .with_scale(Vec3::splat(zoom.zoom)), // Scale text with zoom to maintain screen size
        GridTooltip,
    ));
    
    // Show marker at grid intersection if hovering near one
    if is_near_intersection {
        commands.spawn((
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 0.6, 0.8))),
            Transform::from_xyz(nearest_x, nearest_y, 5.0)
                .with_scale(Vec3::splat(5.0 * zoom.zoom)),
            GridTooltip,
        ));
    }
}

