use bevy::prelude::{Component, Vec2};

use crate::config::*;

pub fn screen_to_world(screen_coord: Vec2) -> Vec2 {
    Vec2::new(
        screen_coord.x - WINDOW_WIDTH as f32 / 2.0,
        screen_coord.y - WINDOW_HEIGHT as f32 / 2.0,
    )
}

use bevy_inspector_egui::Inspectable;

#[derive(Debug)]
pub struct MouseLocation(pub Vec2);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct Bullet;

#[derive(Component, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug)]
pub struct Acceleration(pub Vec2);

#[derive(Component, Inspectable, Default)]
pub struct Tile;

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PlayerState {
    WALKING,
    RUNNING,
    IDLE,
    JUMPING,
}
