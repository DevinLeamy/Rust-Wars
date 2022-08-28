use bevy::prelude::*;

/*
The bottom row is 0 and the top row is GRID_ROWS.
Rows can be above or below zero.
*/

pub const WINDOW_WIDTH: u32 = 920;
pub const WINDOW_HEIGHT: u32 = 920;

// visible grid rows/cols
pub const GRID_ROWS: u32 = 10;
pub const GRID_COLS: u32 = 10;

pub const UNIT_WIDTH: u32 = WINDOW_WIDTH / GRID_COLS;
pub const UNIT_HEIGHT: u32 = WINDOW_HEIGHT / GRID_ROWS;

pub const GRAVITY: f32 = -20.0;

pub const WORLD_ZERO_X: f32 = -(WINDOW_WIDTH as f32 / 2.0);
pub const WORLD_ZERO_Y: f32 = -(WINDOW_HEIGHT as f32 / 2.0);

pub const TIME_STEP: f32 = 1.0 / 60.0; // 60fps

pub const PLAYER_COLOR: Color = Color::CRIMSON;
