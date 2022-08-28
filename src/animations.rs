use std::collections::HashMap;

use benimator::*;
use bevy::prelude::*;

#[derive(Clone)]
pub struct Animation {
    texture_atlas: Handle<TextureAtlas>,
    animation: Handle<SpriteSheetAnimation>,
}

impl Animation {
    pub fn new(
        texture_atlas: &Handle<TextureAtlas>,
        animation: &Handle<SpriteSheetAnimation>,
    ) -> Self {
        Animation {
            texture_atlas: texture_atlas.clone(),
            animation: animation.clone(),
        }
    }

    pub fn get_texture_atlas_handle(self) -> Handle<TextureAtlas> {
        self.texture_atlas
    }

    pub fn get_animation_handle(self) -> Handle<SpriteSheetAnimation> {
        self.animation
    }
}

#[derive(Component)]
pub struct Animations {
    animation_map: HashMap<String, Animation>,
}

impl Animations {
    pub fn new() -> Self {
        Animations {
            animation_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, animation: Animation) -> () {
        self.animation_map.insert(name, animation.clone());
    }

    pub fn get_handle(&self, name: String) -> Animation {
        match self.animation_map.get(&name) {
            Some(animation) => animation.clone(),
            None => panic!("Error: trying to use unregistered animation"),
        }
    }
}

/*
 * Add an animation map to the player component, initialize it with
 * animations and then access those animations when updating the player movement
 */
