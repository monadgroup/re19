use super::animation_clip::AnimationClip;
use super::property::PropertyValue;
use super::schema::GeneratorSchema;
use crate::generator::Generator;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Default)]
pub struct Timeline {
    pub tracks: Vec<Track>,
}

#[derive(Default)]
pub struct Track {
    pub clips: Vec<Clip>,
}

pub enum ClipSource {
    Generator(Box<dyn Generator>),
    Animation(AnimationClip),
}

impl ClipSource {
    pub fn is_generator(&self) -> bool {
        match self {
            ClipSource::Generator(_) => true,
            _ => false,
        }
    }

    pub fn is_animation(&self) -> bool {
        match self {
            ClipSource::Animation(_) => true,
            _ => false,
        }
    }

    pub fn generator(&self) -> Option<&dyn Generator> {
        match self {
            ClipSource::Generator(gen) => Some(gen.as_ref()),
            _ => None,
        }
    }

    pub fn generator_mut(&mut self) -> Option<&mut dyn Generator> {
        match self {
            ClipSource::Generator(gen) => Some(gen.as_mut()),
            _ => None,
        }
    }

    pub fn animation(&self) -> Option<&AnimationClip> {
        match self {
            ClipSource::Animation(animation) => Some(animation),
            _ => None,
        }
    }

    pub fn animation_mut(&mut self) -> Option<&mut AnimationClip> {
        match self {
            ClipSource::Animation(animation) => Some(animation),
            _ => None,
        }
    }
}

pub struct Clip {
    pub id: u32,
    #[cfg(debug_assertions)]
    pub name: String,
    pub schema: &'static GeneratorSchema,
    pub source: ClipSource,

    pub offset_frames: u32,
    pub duration_frames: u32,

    pub property_groups: Vec<PropertyGroup>,
    pub is_selected: bool,
}

pub struct PropertyGroup {
    pub defaults: Vec<PropertyDefault>,
}

pub struct PropertyDefault {
    pub value: PropertyValue,
    pub is_override: bool,
}
