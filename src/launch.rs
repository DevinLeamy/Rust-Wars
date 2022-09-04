use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{GameState, Global};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_menu.run_in_state(GameState::Menu));
    }
}

fn update_menu(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut global: ResMut<Global>,
) {
    let play = keyboard_input.just_pressed(KeyCode::Space);

    if play {
        global.start_playing();
        commands.insert_resource(NextState(GameState::LoadWaveState));
    }
}
