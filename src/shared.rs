use benimator::FrameRate;
use bevy::prelude::*;
use rand::random;
use std::{collections::HashMap, time::Duration};

use crate::{aliens::{Rylo, Aris, Zorg}, player::SHIP_BULLET_SIZE};

pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const CAMERA_LEVEL: f32 = 1.0;

// window
pub const WINDOW_WIDTH: f32 = 920.;
pub const WINDOW_HEIGHT: f32 = 920.;

// walls
pub const WALL_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
pub const BOTTOM_WALL: f32 = -WINDOW_HEIGHT / 2.;
pub const TOP_WALL: f32 = WINDOW_HEIGHT / 2.;
pub const LEFT_WALL: f32 = -WINDOW_WIDTH / 2.;
pub const RIGHT_WALL: f32 = WINDOW_WIDTH / 2.;
pub const WALL_THICKNESS: f32 = 10.;

// bullet
pub const BULLET_SIZE: Vec2 = Vec2::new(20.0, 40.0);
pub const SHIP_BULLET_SPEED: f32 = 350.0;
pub const SHIP_BULLET_INITIAL_GAP: f32 = 5.;
pub const BULLET_LAYER: f32 = 1.0;

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

#[derive(Component, Deref, DerefMut)]
pub struct DespawnTimer(pub Timer);

impl DespawnTimer {
    pub fn from_seconds(duration: f32) -> DespawnTimer {
        DespawnTimer(Timer::from_seconds(duration, false))
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
pub struct Health(pub u32);

#[derive(Component, Deref)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, PartialEq)]
pub enum Bullet {
    Ship,
    Alien,
}

pub trait DurationGenerator {
    fn sample(&self) -> Duration; 
}

pub struct AtMost(pub f32);

impl DurationGenerator for AtMost {
    fn sample(&self) -> Duration {
        duration_at_most(self.0)
    }
}
pub struct Between(pub f32, pub f32); 

impl DurationGenerator for Between {
    fn sample(&self) -> Duration {
        duration_between(self.0, self.1)
    }
}

pub struct Fixed(pub f32);

impl DurationGenerator for Fixed {
    fn sample(&self) -> Duration {
        Duration::from_secs_f32(self.0)
    }
}

pub enum DurationType {
    AtMost(AtMost),
    Between(Between),
    Fixed(Fixed)
}

impl DurationGenerator for DurationType {
    fn sample(&self) -> Duration {
        match &self {
            DurationType::AtMost(gen)  => gen.sample(),
            DurationType::Between(gen) => gen.sample(),
            DurationType::Fixed(gen)   => gen.sample()
        }
    }
}

#[derive(Component)]
pub struct ShootingCooldown {
    timer: Timer,
    duration: DurationType 
}

impl ShootingCooldown {
    pub fn new(duration: DurationType) -> Self {
        ShootingCooldown {
            timer: Timer::new(duration.sample(), false),
            duration 
        } 
    }

    pub fn tick(&mut self, delta: f32) {
        self.timer.tick(Duration::from_secs_f32(delta));
    }

    pub fn finished(&self) -> bool {
        self.timer.finished()
    }

    pub fn reset(&mut self) {
        self.timer.set_duration(self.duration.sample());
        self.timer.reset();
    }
}

pub struct Sprites {
    sprites: HashMap<String, Handle<Image>>,
}

impl Sprites {
    pub fn new() -> Self {
        Sprites {
            sprites: HashMap::default(),
        }
    }
    pub fn add(&mut self, sprite_name: &str, sprite: Handle<Image>) {
        self.sprites.insert(sprite_name.to_string(), sprite);
    }

    pub fn get(&self, sprite_name: &str) -> Handle<Image> {
        self.sprites.get(sprite_name).expect(format!("Sprite ({}) not found", sprite_name).as_str()).clone()
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    bullet: Bullet,
    collider: Collider,
    velocity: Velocity,
}

impl BulletBundle {
    pub fn new(
        translation: Vec2, 
        sprite: Handle<Image>,
        size: Vec2,
        velocity: Velocity,
        rotation: f32,
        bullet_type: Bullet 
    ) -> BulletBundle {
        BulletBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(BULLET_LAYER),
                    rotation: Quat::from_rotation_z(rotation.to_radians()),
                    ..default()
                },
                texture: sprite,
                sprite: Sprite {
                    custom_size: Some(size),
                    ..default()
                },
                ..default()
            },
            velocity,
            bullet: bullet_type, 
            collider: Collider { size },
        } 
    }

    pub fn from_aris(translation: Vec2, sprite: Handle<Image>) -> BulletBundle {
        BulletBundle::new(
            translation, 
            sprite, 
            BULLET_SIZE, 
            Velocity(Vec2::new(0., -Aris::BULLET_SPEED)), 
            0.0,
            Bullet::Alien
        )
    }

    pub fn from_rylo(translation: Vec2, sprite: Handle<Image>) -> BulletBundle {
        BulletBundle::new(
            translation, 
            sprite, 
            BULLET_SIZE, 
            Velocity(Vec2::new(0., -Rylo::BULLET_SPEED)), 
            0.0,
            Bullet::Alien
        )
    }

    pub fn from_zorg(translation: Vec2, sprite: Handle<Image>, velocity: Velocity, rotation: f32) -> BulletBundle {
        BulletBundle::new(
            translation, 
            sprite, 
            Zorg::BULLET_SIZE, 
            velocity,
            rotation,
            Bullet::Alien
        ) 
    }

    pub fn from_ship(translation: Vec2, sprite: Handle<Image>) -> BulletBundle {
        BulletBundle::new(
            translation, 
            sprite, 
            SHIP_BULLET_SIZE, 
            Velocity(Vec2::new(0., SHIP_BULLET_SPEED)), 
            0.0,
            Bullet::Ship
        )
    }
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

#[derive(Component, Deref, Clone)]
pub struct BAnimation(pub benimator::Animation);

#[derive(Clone)]
pub enum ImageData {
    TextureAtlas(Handle<TextureAtlas>),
    Images(Vec<String>),
}

#[derive(Bundle)]
pub struct AnimationBundle {
    animation: BAnimation,
    animation_state: AnimationState 
}

impl AnimationBundle {
    pub fn from_animation(animation: Animation) -> AnimationBundle {
        AnimationBundle {
            animation: animation.animation,
            animation_state: AnimationState::default()
        }
    }
}

// TODO: use readonly public crate
#[derive(Clone)]
pub struct Animation {
    pub animation: BAnimation,
    pub image_data: ImageData,
}

impl Animation {
    pub fn from_images(images: Vec<String>, frame_duration: u64) -> Animation {
        Animation {
            animation: BAnimation(benimator::Animation::from_indices(
                0..images.len(),
                FrameRate::from_frame_duration(Duration::from_millis(frame_duration)),
            )),
            image_data: ImageData::Images(images)
        }
    }

    pub fn from_texture(atlas: Handle<TextureAtlas>, images: u32, frame_duration: u64) -> Animation {
        Animation {
            animation: BAnimation(benimator::Animation::from_indices(
                0..images as usize,
                FrameRate::from_frame_duration(Duration::from_millis(frame_duration)),
            )),
            image_data: ImageData::TextureAtlas(atlas),
        }
    }
}

pub struct Animations {
    animations: HashMap<String, Animation>,
}

impl Animations {
    pub fn new() -> Self {
        Animations {
            animations: HashMap::default(),
        }
    }
    pub fn add(&mut self, animation_name: &str, animation: Animation) {
        self.animations.insert(animation_name.to_string(), animation);
    }

    pub fn get(&self, animation_name: &str) -> Animation {
        self.animations.get(animation_name).unwrap().clone()
    }
}

#[derive(Bundle)]
pub struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
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
            WallLocation::Left | WallLocation::Right => Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT),
            WallLocation::Bottom | WallLocation::Top => Vec2::new(WINDOW_WIDTH, WALL_THICKNESS),
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
        }
    }
}

pub fn duration_between(min_time: f32, max_time: f32) -> Duration {
    let duration = min_time + random::<f32>() * (max_time - min_time);
    Duration::from_secs_f32(duration)
}

pub fn duration_at_most(max_time: f32) -> Duration {
    let duration = random::<f32>() * max_time;
    Duration::from_secs_f32(duration)
}

pub fn update_shooting_cooldowns(mut query: Query<&mut ShootingCooldown>) {
    for mut cooldown in query.iter_mut() {
        cooldown.tick(TIME_STEP);
    }
}
