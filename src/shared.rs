use bevy::prelude::*;
use std::{time::Duration, collections::HashMap};

use crate::{player::{SHIP_BULLET_IMAGE_SIZE, SHIP_BULLET_SIZE}, aliens::{ALIEN_BULLET_COLOR, ALIEN_BULLET_SPEED}};

pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const CAMERA_LEVEL: f32 = 1.0;

// window 
pub const WINDOW_WIDTH: f32 = 920.;
pub const WINDOW_HEIGHT: f32 = 920.;

// walls
pub const WALL_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
pub const BOTTOM_WALL: f32 = -WINDOW_HEIGHT / 2.;
pub const TOP_WALL: f32 = WINDOW_HEIGHT / 2.;
pub const LEFT_WALL: f32 = -WINDOW_WIDTH / 2.;
pub const RIGHT_WALL: f32 = WINDOW_WIDTH / 2.;
pub const WALL_THICKNESS: f32 = 10.;

// bullet 
pub const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);
pub const SHIP_BULLET_SPEED: f32 = 350.0;
pub const SHIP_BULLET_INITIAL_GAP: f32 = 5.;

// background
pub const BACKGROUND_FONT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
pub const BACKGROUND_FONT_SIZE: f32 = 30.0;
pub const BACKGROUND_LEVEL: f32 = -1.0;

// scoreboard
pub const SCORE_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
pub const SCOREBOARD_FONT_SIZE: f32 = 30.0;
pub const SCOREBOARD_PADDING_TOP: Val = Val::Px(8.0);
pub const SCOREBOARD_PADDING_LEFT: Val = Val::Px(10.0);

// explosion
pub const EXPLOSION_SIZE: f32 = 0.3;
pub const EXPLOSION_FRAME_DURATION_IN_MILLIS: u64 = 20;

#[derive(Component)]
pub struct Collider;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, PartialEq)]
pub enum Bullet {
    Ship,
    Alien
}

#[derive(Component, Deref, DerefMut)]
pub struct ShootingCooldown(pub Timer);


pub struct Sprites {
    sprites: HashMap<String, Handle<Image>>
}

impl Sprites {
    pub fn new() -> Self {
        Sprites {
            sprites: HashMap::default()
        }
    }
    pub fn add(&mut self, sprite_name: String, sprite: Handle<Image>) {
        self.sprites.insert(sprite_name, sprite);
    }
    pub fn get(&self, sprite_name: String) -> Option<&Handle<Image>> {
        self.sprites.get(&sprite_name)
    }
}


#[derive(Bundle)]
pub struct BulletBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    bullet: Bullet,
    collider: Collider,
    velocity: Velocity
}

impl BulletBundle {
    pub fn from_alien(translation: Vec2) -> BulletBundle {
        BulletBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: BULLET_SIZE.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: ALIEN_BULLET_COLOR,
                    ..default()
                },
                ..default()
            },
            velocity: Velocity(Vec2::new(0.0, -ALIEN_BULLET_SPEED)),
            bullet: Bullet::Alien,
            collider: Collider,
        }
    }

    pub fn from_ship(translation: Vec2, sprite: Handle<Image>) -> BulletBundle {
        BulletBundle {
            sprite_bundle: SpriteBundle {
                texture: sprite,
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: SHIP_BULLET_SIZE.extend(1.0),
                    ..default()
                },
                sprite: generate_texture_sprite(SHIP_BULLET_SIZE, SHIP_BULLET_IMAGE_SIZE), 
                ..default()
            },
            velocity: Velocity(Vec2::new(0.0, SHIP_BULLET_SPEED)),
            bullet: Bullet::Ship,
            collider: Collider,
        }
    }
}

pub fn generate_texture_sprite(entity_size: Vec2, texture_size: Vec2) -> Sprite {
    let size_x = entity_size.x / texture_size.x * 10.0;
    let size_y = entity_size.y / texture_size.y * 10.0;

    Sprite {
        custom_size: Some(Vec2::new(size_x, size_y) * Vec2::splat(1.4)),
        ..default()
    }
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

#[derive(Component, Deref, Clone)]
pub struct BAnimation(pub benimator::Animation);

pub enum ImageData {
    TextureAtlas(Handle<TextureAtlas>),
    Images(Vec<String>)
}

// TODO: use readonly public crate
pub struct Animation {
    pub animation: BAnimation,
    pub image_data: ImageData,
}

pub struct Animations {
    animations: HashMap<String, Animation>
}

impl Animations {
    pub fn new() -> Self {
        Animations {
            animations: HashMap::default()
        }
    }
    pub fn add(&mut self, animation_name: String, animation: Animation) {
        self.animations.insert(animation_name, animation);
    }
    pub fn get(&self, animation_name: String) -> Option<&Animation> {
        self.animations.get(&animation_name)
    }
}


#[derive(Bundle)]
pub struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}


pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(WINDOW_WIDTH, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    pub fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}
