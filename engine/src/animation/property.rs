use super::clip::ClipReference;
use crate::animation::clip::ClipPropertyValue;
use crate::math::{Quaternion, RgbColor, RgbaColor, Vector2, Vector3, Vector4};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    RgbColor,
    RgbaColor,
    Rotation,
    ClipReference,
}

impl PropertyType {
    pub fn num_fields(self) -> usize {
        match self {
            PropertyType::Float => 1,
            PropertyType::Vec2 => 2,
            PropertyType::Vec3 => 3,
            PropertyType::Vec4 => 4,
            PropertyType::RgbColor => 3,
            PropertyType::RgbaColor => 4,
            PropertyType::Rotation => 3,
            PropertyType::ClipReference => 1,
        }
    }

    pub fn default_value(self) -> PropertyValue {
        match self {
            PropertyType::Float => PropertyValue::Float(0.),
            PropertyType::Vec2 => PropertyValue::Vec2(Vector2::default()),
            PropertyType::Vec3 => PropertyValue::Vec3(Vector3::default()),
            PropertyType::Vec4 => PropertyValue::Vec4(Vector4::default()),
            PropertyType::RgbColor => PropertyValue::RgbColor(RgbColor::new(1., 0., 1.)),
            PropertyType::RgbaColor => PropertyValue::RgbaColor(RgbaColor::new(1., 0., 1., 1.)),
            PropertyType::Rotation => PropertyValue::Rotation(Quaternion::default()),
            PropertyType::ClipReference => PropertyValue::ClipReference(None),
        }
    }

    pub fn value_range(self) -> Option<(f32, f32)> {
        match self {
            PropertyType::Float
            | PropertyType::Vec2
            | PropertyType::Vec3
            | PropertyType::Vec4
            | PropertyType::ClipReference => None,
            PropertyType::RgbColor | PropertyType::RgbaColor => Some((0., 1.)),
            PropertyType::Rotation => Some((-180., 180.)),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PropertyValue {
    Float(f32),
    Vec2(Vector2),
    Vec3(Vector3),
    Vec4(Vector4),
    RgbColor(RgbColor),
    RgbaColor(RgbaColor),
    Rotation(Quaternion),
    ClipReference(Option<ClipReference>),
}

fn float_from_iter(fields: &mut dyn Iterator<Item = f32>) -> Option<f32> {
    fields.next()
}

fn vec2_from_iter(fields: &mut dyn Iterator<Item = f32>) -> Option<Vector2> {
    fields
        .next()
        .and_then(|x| fields.next().map(|y| Vector2 { x, y }))
}

fn vec3_from_iter(fields: &mut dyn Iterator<Item = f32>) -> Option<Vector3> {
    fields.next().and_then(|x| {
        fields
            .next()
            .and_then(|y| fields.next().map(|z| Vector3 { x, y, z }))
    })
}

fn vec4_from_iter(fields: &mut dyn Iterator<Item = f32>) -> Option<Vector4> {
    fields.next().and_then(|x| {
        fields.next().and_then(|y| {
            fields
                .next()
                .and_then(|z| fields.next().map(|w| Vector4 { x, y, z, w }))
        })
    })
}

impl PropertyValue {
    pub fn from_fields(
        prop_type: PropertyType,
        fields: &mut dyn Iterator<Item = f32>,
    ) -> Option<PropertyValue> {
        match prop_type {
            PropertyType::Float => float_from_iter(fields).map(PropertyValue::Float),
            PropertyType::Vec2 => vec2_from_iter(fields).map(PropertyValue::Vec2),
            PropertyType::Vec3 => vec3_from_iter(fields).map(PropertyValue::Vec3),
            PropertyType::Vec4 => vec4_from_iter(fields).map(PropertyValue::Vec4),
            PropertyType::RgbColor => {
                vec3_from_iter(fields).map(|vec| PropertyValue::RgbColor(RgbColor(vec)))
            }
            PropertyType::RgbaColor => {
                vec4_from_iter(fields).map(|vec| PropertyValue::RgbaColor(RgbaColor(vec)))
            }
            PropertyType::Rotation => vec3_from_iter(fields).map(|vec| {
                PropertyValue::Rotation(Quaternion::euler(
                    vec.x.to_radians(),
                    vec.y.to_radians(),
                    vec.z.to_radians(),
                ))
            }),
            PropertyType::ClipReference => float_from_iter(fields).map(|f| {
                PropertyValue::ClipReference(if f < 0. {
                    None
                } else {
                    Some(ClipReference::new(f as u32))
                })
            }),
        }
    }

    pub fn into_float(self) -> Option<f32> {
        match self {
            PropertyValue::Float(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_vec2(self) -> Option<Vector2> {
        match self {
            PropertyValue::Vec2(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_vec3(self) -> Option<Vector3> {
        match self {
            PropertyValue::Vec3(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_vec4(self) -> Option<Vector4> {
        match self {
            PropertyValue::Vec4(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_rgb_color(self) -> Option<RgbColor> {
        match self {
            PropertyValue::RgbColor(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_rgba_color(self) -> Option<RgbaColor> {
        match self {
            PropertyValue::RgbaColor(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_rotation(self) -> Option<Quaternion> {
        match self {
            PropertyValue::Rotation(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_clip_reference(self) -> Option<Option<ClipReference>> {
        match self {
            PropertyValue::ClipReference(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_type(self) -> PropertyType {
        match self {
            PropertyValue::Float(_) => PropertyType::Float,
            PropertyValue::Vec2(_) => PropertyType::Vec2,
            PropertyValue::Vec3(_) => PropertyType::Vec3,
            PropertyValue::Vec4(_) => PropertyType::Vec4,
            PropertyValue::RgbColor(_) => PropertyType::RgbColor,
            PropertyValue::RgbaColor(_) => PropertyType::RgbaColor,
            PropertyValue::Rotation(_) => PropertyType::Rotation,
            PropertyValue::ClipReference(_) => PropertyType::ClipReference,
        }
    }

    pub fn lerp(self, other: PropertyValue, amount: f32) -> Option<PropertyValue> {
        match self {
            PropertyValue::Float(a) => other
                .into_float()
                .map(|b| PropertyValue::Float(a + (b - a) * amount)),
            PropertyValue::Vec2(a) => other
                .into_vec2()
                .map(|b| PropertyValue::Vec2(a.lerp(b, amount))),
            PropertyValue::Vec3(a) => other
                .into_vec3()
                .map(|b| PropertyValue::Vec3(a.lerp(b, amount))),
            PropertyValue::Vec4(a) => other
                .into_vec4()
                .map(|b| PropertyValue::Vec4(a.lerp(b, amount))),
            PropertyValue::RgbColor(a) => other
                .into_rgb_color()
                .map(|b| PropertyValue::RgbColor(a.lerp(b, amount))),
            PropertyValue::RgbaColor(a) => other
                .into_rgba_color()
                .map(|b| PropertyValue::RgbaColor(a.lerp(b, amount))),
            PropertyValue::Rotation(a) => other
                .into_rotation()
                .map(|b| PropertyValue::Rotation(a.slerp(b, amount))),
            PropertyValue::ClipReference(a) => other
                .into_clip_reference()
                .map(|b| PropertyValue::ClipReference(if amount >= 1. { b } else { a })),
        }
    }

    pub fn fields(self) -> PropertyValueIter {
        // Convert rotations (quaternions) into euler angles for fields
        let iter_val = if let PropertyValue::Rotation(rot) = self {
            let euler_rads = rot.as_euler();
            PropertyValue::Vec3(
                (
                    euler_rads.0.to_degrees(),
                    euler_rads.1.to_degrees(),
                    euler_rads.2.to_degrees(),
                )
                    .into(),
            )
        } else {
            self
        };

        PropertyValueIter {
            val: iter_val,
            index: 0,
        }
    }
}

impl From<f32> for PropertyValue {
    fn from(val: f32) -> PropertyValue {
        PropertyValue::Float(val)
    }
}

impl From<Vector2> for PropertyValue {
    fn from(val: Vector2) -> PropertyValue {
        PropertyValue::Vec2(val)
    }
}

impl From<Vector3> for PropertyValue {
    fn from(val: Vector3) -> PropertyValue {
        PropertyValue::Vec3(val)
    }
}

impl From<Vector4> for PropertyValue {
    fn from(val: Vector4) -> PropertyValue {
        PropertyValue::Vec4(val)
    }
}

impl From<RgbColor> for PropertyValue {
    fn from(val: RgbColor) -> PropertyValue {
        PropertyValue::RgbColor(val)
    }
}

impl From<RgbaColor> for PropertyValue {
    fn from(val: RgbaColor) -> PropertyValue {
        PropertyValue::RgbaColor(val)
    }
}

impl From<Quaternion> for PropertyValue {
    fn from(val: Quaternion) -> PropertyValue {
        PropertyValue::Rotation(val)
    }
}

impl From<Option<ClipReference>> for PropertyValue {
    fn from(val: Option<ClipReference>) -> PropertyValue {
        PropertyValue::ClipReference(val)
    }
}

pub struct PropertyValueIter {
    val: PropertyValue,
    index: usize,
}

impl PropertyValueIter {
    pub fn len(&self) -> usize {
        self.val.get_type().num_fields()
    }
}

impl Iterator for PropertyValueIter {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let index = self.index;
        self.index += 1;

        match self.val {
            PropertyValue::Float(val) => match index {
                0 => Some(val),
                _ => None,
            },
            PropertyValue::Vec2(val) => match index {
                0 => Some(val.x),
                1 => Some(val.y),
                _ => None,
            },
            PropertyValue::Vec3(val) => match index {
                0 => Some(val.x),
                1 => Some(val.y),
                2 => Some(val.z),
                _ => None,
            },
            PropertyValue::Vec4(val) => match index {
                0 => Some(val.x),
                1 => Some(val.y),
                2 => Some(val.z),
                3 => Some(val.w),
                _ => None,
            },
            PropertyValue::RgbColor(val) => match index {
                0 => Some(val.r()),
                1 => Some(val.g()),
                2 => Some(val.b()),
                _ => None,
            },
            PropertyValue::RgbaColor(val) => match index {
                0 => Some(val.r()),
                1 => Some(val.g()),
                2 => Some(val.b()),
                3 => Some(val.a()),
                _ => None,
            },
            PropertyValue::ClipReference(val) => match index {
                0 => Some(match val {
                    Some(val) => val.clip_id() as f32,
                    None => -1.,
                }),
                _ => None,
            },

            // Note: this should never happen, since when creating the iter
            // quaternions are converted into euler angles (which are vec3)
            PropertyValue::Rotation(_) => unreachable!(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

pub trait Gettable {
    fn get(val: PropertyValue) -> Self;
}

impl Gettable for f32 {
    fn get(val: PropertyValue) -> f32 {
        val.into_float().unwrap()
    }
}
impl Gettable for Vector2 {
    fn get(val: PropertyValue) -> Vector2 {
        val.into_vec2().unwrap()
    }
}
impl Gettable for Vector3 {
    fn get(val: PropertyValue) -> Vector3 {
        val.into_vec3().unwrap()
    }
}
impl Gettable for Vector4 {
    fn get(val: PropertyValue) -> Vector4 {
        val.into_vec4().unwrap()
    }
}
impl Gettable for RgbColor {
    fn get(val: PropertyValue) -> RgbColor {
        val.into_rgb_color().unwrap()
    }
}
impl Gettable for RgbaColor {
    fn get(val: PropertyValue) -> RgbaColor {
        val.into_rgba_color().unwrap()
    }
}
impl Gettable for Quaternion {
    fn get(val: PropertyValue) -> Quaternion {
        val.into_rotation().unwrap()
    }
}
impl Gettable for Option<ClipReference> {
    fn get(val: PropertyValue) -> Option<ClipReference> {
        val.into_clip_reference().unwrap()
    }
}

pub fn prop<T: Gettable>(properties: &[&[ClipPropertyValue]], group: usize, prop: usize) -> T {
    T::get(properties[group][prop].value)
}
