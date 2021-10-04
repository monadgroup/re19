use super::property::PropertyValue;
use crate::generator::Generator;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipReference {
    clip_id: u32,
}

impl ClipReference {
    pub fn new(clip_id: u32) -> Self {
        ClipReference { clip_id }
    }

    pub fn clip_id(self) -> u32 {
        self.clip_id
    }
}

pub struct ClipPropertyValue {
    pub value: PropertyValue,
    pub is_overridden: bool,
    pub targeted_by: Option<ClipReference>,
}

pub struct ActiveClip {
    pub name: String,
    pub reference: ClipReference,
    pub track_index: usize,
    pub clip_index: usize,
    pub local_time: u32,
    pub properties: Vec<Vec<ClipPropertyValue>>,
}

pub trait ActiveClipMap {
    fn active_clips(&self) -> &[ActiveClip];
    fn active_clips_mut(&mut self) -> &mut [ActiveClip];
    fn get_clip_index(&self, reference: ClipReference) -> Option<usize>;
}

pub trait GeneratorClipMap {
    fn try_get_clip(&self, reference: ClipReference) -> Option<&dyn Generator>;
    fn try_get_clip_mut(&mut self, reference: ClipReference) -> Option<&mut dyn Generator>;

    fn get_clip(&self, reference: ClipReference) -> &dyn Generator {
        self.try_get_clip(reference).unwrap()
    }
    fn get_clip_mut(&mut self, reference: ClipReference) -> &mut dyn Generator {
        self.try_get_clip_mut(reference).unwrap()
    }
}
