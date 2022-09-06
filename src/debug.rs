use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use iyes_loopless::state::NextState;

use crate::{shared::{Collider, Health}, aliens::Alien, GameState, Global};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    #[cfg(feature = "debug")]
    fn build(&self, app: &mut App) {
        println!("Debugging enabled");
        app.add_plugin(WorldInspectorPlugin::new())
            .add_system(draw_bounding_boxes)
            .add_system(goto_next_wave);
    }
    #[cfg(not(feature = "debug"))]
    fn build(&self, app: &mut App) {}
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

fn goto_next_wave(
    keyboard_input: Res<Input<KeyCode>>, 
    mut commands: Commands, 
    query: Query<Entity, With<Alien>>,
    mut global: ResMut<Global>
) {
    if keyboard_input.just_pressed(KeyCode::N) {
        for alien in query.iter() {
            commands.entity(alien).despawn_recursive();
        }

        global.wave_cleared();

        commands.insert_resource(NextState(GameState::LoadWaveState));
    }
}
