use bevy::prelude::*;

#[derive(Clone, Copy)]
pub struct Bounds {
    /// bottom left world coordinates
    pub position: Vec2,
    /// width and height in world coordinates
    pub size: Vec2,
}

impl Bounds {
    /// check if 2d world coordinate is within bounds
    pub fn is_in_bounds(&self, x: f32, y: f32) -> bool {
        if x < self.position.x || x > self.position.x + self.size.x {
            false
        } else if y < self.position.y || y > self.position.y + self.size.y {
            false
        } else {
            true
        }
    }
}
