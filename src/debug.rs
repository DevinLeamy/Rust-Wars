use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::shared::{Collider, Health};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    #[cfg(feature = "debug")]
    fn build(&self, app: &mut App) {
        println!("Debugging enabled");
        app.add_plugin(WorldInspectorPlugin::new())
            .add_system(draw_bounding_boxes)
            .add_system(debug_ship);
    }
    #[cfg(not(feature = "debug"))]
    fn build(&self, app: &mut App) {}
}

fn debug_ship(ship_query: Query<(&Transform, &Health)>) {
    // if let Ok((_transform, health)) = ship_query.get_single() {
    // }
}

fn draw_bounding_boxes(mut commands: Commands, query: Query<(Entity, &Collider), Added<Collider>>) {
    for (entity, collider) in query.iter() {
        let bounding_box = commands
            .spawn()
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec2::ZERO.extend(-0.1),
                    scale: collider.size.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Bounding Box"))
            .id();

        commands.entity(entity).add_child(bounding_box);
    }
}
