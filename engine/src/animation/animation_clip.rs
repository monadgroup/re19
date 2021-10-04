use super::clip::ClipReference;
use super::cubic_bezier::CubicBezier;
use super::property::PropertyValue;
use alloc::vec::Vec;

pub struct AnimationClip {
    pub target_clip: ClipReference,
    //pub time_property: AnimatedPropertyField,
    pub properties: Vec<AnimatedProperty>,
    //pub is_time_collapsed: bool,
}

pub struct AnimatedProperty {
    pub group_index: usize,
    pub property_index: usize,
    pub target: AnimatedPropertyTarget,
    pub is_collapsed: bool,
}

pub enum AnimatedPropertyTarget {
    Joined(AnimatedPropertyField),
    Separate(Vec<AnimatedPropertyField>),
}

pub struct AnimatedPropertyField {
    pub local_offset_frames: i32,
    pub start_value: PropertyValue,
    pub segments: Vec<CurveSegment>,
}

pub struct CurveSegment {
    pub duration_frames: u32,
    pub end_value: PropertyValue,
    pub interpolation: CurveInterpolation,
}

pub enum CurveInterpolation {
    Linear,
    CubicBezier(CubicBezier),
}

impl CurveInterpolation {
    pub fn eval(&self, t: f32) -> f32 {
        match self {
            CurveInterpolation::Linear => t,
            CurveInterpolation::CubicBezier(bezier) => bezier.get_y_at(t),
        }
    }

    pub fn is_linear(&self) -> bool {
        match self {
            CurveInterpolation::Linear => true,
            _ => false,
        }
    }

    pub fn is_cubic_bezier(&self) -> bool {
        match self {
            CurveInterpolation::CubicBezier(_) => true,
            _ => false,
        }
    }
}
