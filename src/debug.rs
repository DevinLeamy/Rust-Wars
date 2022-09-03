use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::shared::Health;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WorldInspectorPlugin::new())
            .add_system(debug_ship);
    }
}

fn debug_ship(
    ship_query: Query<(&Transform, &Health)>
) {
    // if let Ok((_transform, health)) = ship_query.get_single() {
    // }
}

