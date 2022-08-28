use bevy::prelude::*;

use crate::utils::*;

pub fn mouse_position_system(
    mut mouse_position: ResMut<MouseLocation>,
    // mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    // mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in cursor_moved_events.iter() {
        mouse_position.0 = screen_to_world(event.position);
    }
}
