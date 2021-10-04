use engine::animation::schema::GeneratorSchema;
use engine::animation::{animation_clip, clip, cubic_bezier, property, schema, timeline};
use engine::creation_context::CreationContext;
use engine::generator::GENERATOR_SCHEMAS;
use engine::math;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::iter::FromIterator;

pub fn serialize_timeline<S: Serializer>(
    timeline: &timeline::Timeline,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let timeline = Timeline::from(timeline);
    timeline.serialize(serializer)
}

pub fn deserialize_timeline<'de, D: Deserializer<'de>>(
    deserializer: D,
    context: &mut CreationContext,
) -> Result<timeline::Timeline, D::Error> {
    Ok(Timeline::deserialize(deserializer)?.into(context))
}

#[derive(Serialize, Deserialize)]
struct Timeline {
    pub tracks: Vec<Track>,
}

impl From<&timeline::Timeline> for Timeline {
    fn from(timeline: &timeline::Timeline) -> Self {
        Timeline {
            tracks: timeline
                .tracks
                .iter()
                .map(|track| Track::from(track))
                .collect(),
        }
    }
}

impl Timeline {
    fn into(self, context: &mut CreationContext) -> timeline::Timeline {
        timeline::Timeline {
            tracks: self
                .tracks
                .into_iter()
                .map(|track| track.into(context))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Track {
    pub clips: Vec<Clip>,
}

impl From<&timeline::Track> for Track {
    fn from(track: &timeline::Track) -> Self {
        Track {
            clips: track.clips.iter().map(|clip| Clip::from(clip)).collect(),
        }
    }
}

impl Track {
    fn into(self, context: &mut CreationContext) -> timeline::Track {
        let mut clips = Vec::new();
        let mut next_offset = 0;
        for clip in self.clips {
            match clip.into(context) {
                ConvertedClip::Clip(mut converted_clip) => {
                    converted_clip.offset_frames += next_offset;
                    next_offset = 0;
                    clips.push(converted_clip);
                }
                ConvertedClip::NoClip(offset) => next_offset += offset,
            }
        }

        timeline::Track { clips }
    }
}

#[derive(Serialize, Deserialize)]
struct Clip {
    pub id: u32,
    pub name: String,
    pub schema: String,
    pub animation: Option<AnimationClip>,
    pub offset_frames: u32,
    pub duration_frames: u32,
    pub property_groups: Vec<PropertyGroup>,
}

impl From<&timeline::Clip> for Clip {
    fn from(clip: &timeline::Clip) -> Self {
        Clip {
            id: clip.id,
            name: clip.name.clone(),
            schema: clip.schema.name.to_string(),
            animation: match &clip.source {
                timeline::ClipSource::Animation(animation_clip) => {
                    Some(AnimationClip::from(animation_clip, clip.schema))
                }
                _ => None,
            },
            offset_frames: clip.offset_frames,
            duration_frames: clip.duration_frames,
            property_groups: clip
                .property_groups
                .iter()
                .zip(clip.schema.groups.iter())
                .map(|(property_group, schema)| PropertyGroup::from(property_group, schema))
                .collect(),
        }
    }
}

enum ConvertedClip {
    Clip(timeline::Clip),
    NoClip(u32),
}

impl Clip {
    fn into(self, context: &mut CreationContext) -> ConvertedClip {
        // Figure out which schema we're referencing by searching the available schemas
        let named_schema = GENERATOR_SCHEMAS
            .iter()
            .find(|schema| schema.name == self.schema);
        let named_schema = match named_schema {
            Some(schema) => schema,
            None => {
                eprintln!(
                    "Couldn't find schema \"{}\", so clip \"{}\" will be deleted.",
                    self.schema, self.name
                );
                return ConvertedClip::NoClip(self.offset_frames + self.duration_frames);
            }
        };

        let (generator, property_groups) = match self.animation {
            Some(animation_clip) => (
                timeline::ClipSource::Animation(animation_clip.into(named_schema)),
                Vec::new(),
            ),
            None => {
                let generator =
                    timeline::ClipSource::Generator((named_schema.instantiate_generator)(context));
                let available_groups: HashMap<&str, &PropertyGroup> = HashMap::from_iter(
                    self.property_groups
                        .iter()
                        .map(|group| (&group.name as &str, group)),
                );
                let property_groups = named_schema.groups.iter().map(|schema_group| {
                    match available_groups.get(&schema_group.name) {
                        Some(group) => (*group).into(schema_group),
                        None => schema_group.instantiate(),
                    }
                });

                (generator, property_groups.collect())
            }
        };

        ConvertedClip::Clip(timeline::Clip {
            id: self.id,
            name: self.name,
            schema: named_schema,
            source: generator,
            offset_frames: self.offset_frames,
            duration_frames: self.duration_frames,
            property_groups,
            is_selected: false,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct PropertyGroup {
    pub name: String,
    pub defaults: Vec<PropertyDefault>,
}

impl PropertyGroup {
    fn from(property_group: &timeline::PropertyGroup, schema: &schema::SchemaGroup) -> Self {
        PropertyGroup {
            name: schema.name.to_string(),
            defaults: property_group
                .defaults
                .iter()
                .zip(schema.properties.iter())
                .map(|(property_default, schema)| PropertyDefault::from(property_default, schema))
                .collect(),
        }
    }

    fn into(&self, schema: &schema::SchemaGroup) -> timeline::PropertyGroup {
        let available_defaults: HashMap<&str, &PropertyDefault> = HashMap::from_iter(
            self.defaults
                .iter()
                .map(|default| (&default.name as &str, default)),
        );
        let defaults = schema.properties.iter().map(|schema_prop| {
            match available_defaults.get(&schema_prop.name) {
                Some(default) => (*default).into(),
                None => schema_prop.instantiate(),
            }
        });

        timeline::PropertyGroup {
            defaults: defaults.collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PropertyDefault {
    pub name: String,
    pub value: PropertyValue,
    pub is_override: bool,
}

impl PropertyDefault {
    fn from(property_default: &timeline::PropertyDefault, schema: &schema::SchemaProperty) -> Self {
        PropertyDefault {
            name: schema.name.to_string(),
            value: PropertyValue::from(property_default.value),
            is_override: property_default.is_override,
        }
    }
}

impl Into<timeline::PropertyDefault> for &PropertyDefault {
    fn into(self) -> timeline::PropertyDefault {
        timeline::PropertyDefault {
            value: (&self.value).into(),
            is_override: self.is_override,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum PropertyValue {
    Float(f32),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
    Vec4 { x: f32, y: f32, z: f32, w: f32 },
    RgbColor { r: f32, g: f32, b: f32 },
    RgbaColor { r: f32, g: f32, b: f32, a: f32 },
    Rotation { x: f32, y: f32, z: f32, w: f32 },
    ClipReference(Option<u32>),
}

impl From<property::PropertyValue> for PropertyValue {
    fn from(prop_val: property::PropertyValue) -> Self {
        match prop_val {
            property::PropertyValue::Float(val) => PropertyValue::Float(val),
            property::PropertyValue::Vec2(val) => PropertyValue::Vec2 { x: val.x, y: val.y },
            property::PropertyValue::Vec3(val) => PropertyValue::Vec3 {
                x: val.x,
                y: val.y,
                z: val.z,
            },
            property::PropertyValue::Vec4(val) => PropertyValue::Vec4 {
                x: val.x,
                y: val.y,
                z: val.z,
                w: val.w,
            },
            property::PropertyValue::RgbColor(val) => PropertyValue::RgbColor {
                r: val.r(),
                g: val.g(),
                b: val.b(),
            },
            property::PropertyValue::RgbaColor(val) => PropertyValue::RgbaColor {
                r: val.r(),
                g: val.g(),
                b: val.b(),
                a: val.a(),
            },
            property::PropertyValue::Rotation(val) => PropertyValue::Rotation {
                x: val.x,
                y: val.y,
                z: val.z,
                w: val.w,
            },
            property::PropertyValue::ClipReference(val) => {
                PropertyValue::ClipReference(val.map(|clip_ref| clip_ref.clip_id()))
            }
        }
    }
}

impl Into<property::PropertyValue> for &PropertyValue {
    fn into(self) -> property::PropertyValue {
        match self {
            PropertyValue::Float(val) => property::PropertyValue::Float(*val),
            PropertyValue::Vec2 { x, y } => {
                property::PropertyValue::Vec2(math::Vector2 { x: *x, y: *y })
            }
            PropertyValue::Vec3 { x, y, z } => property::PropertyValue::Vec3(math::Vector3 {
                x: *x,
                y: *y,
                z: *z,
            }),
            PropertyValue::Vec4 { x, y, z, w } => property::PropertyValue::Vec4(math::Vector4 {
                x: *x,
                y: *y,
                z: *z,
                w: *w,
            }),
            PropertyValue::RgbColor { r, g, b } => {
                property::PropertyValue::RgbColor(math::RgbColor::new(*r, *g, *b))
            }
            PropertyValue::RgbaColor { r, g, b, a } => {
                property::PropertyValue::RgbaColor(math::RgbaColor::new(*r, *g, *b, *a))
            }
            PropertyValue::Rotation { x, y, z, w } => {
                property::PropertyValue::Rotation(math::Quaternion {
                    x: *x,
                    y: *y,
                    z: *z,
                    w: *w,
                })
            }
            PropertyValue::ClipReference(val) => property::PropertyValue::ClipReference(
                val.map(|clip_ref| clip::ClipReference::new(clip_ref)),
            ),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AnimationClip {
    target_clip: u32,
    //time_property: AnimatedPropertyField,
    properties: Vec<AnimatedProperty>,
}

impl AnimationClip {
    pub fn from(clip: &animation_clip::AnimationClip, schema: &GeneratorSchema) -> Self {
        AnimationClip {
            target_clip: clip.target_clip.clip_id(),
            properties: clip
                .properties
                .iter()
                .map(|prop| AnimatedProperty::from(prop, schema))
                .collect(),
        }
    }

    pub fn into(self, schema: &GeneratorSchema) -> animation_clip::AnimationClip {
        animation_clip::AnimationClip {
            target_clip: clip::ClipReference::new(self.target_clip),
            //time_property: self.time_property.into(),
            //is_time_collapsed: true,
            properties: self
                .properties
                .into_iter()
                .map(|prop| prop.into(schema))
                .filter_map(|prop| prop)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AnimatedProperty {
    group_name: String,
    property_name: String,
    target: AnimatedPropertyTarget,
}

impl AnimatedProperty {
    pub fn from(property: &animation_clip::AnimatedProperty, schema: &GeneratorSchema) -> Self {
        AnimatedProperty {
            group_name: schema.groups[property.group_index].name.to_string(),
            property_name: schema.groups[property.group_index].properties[property.property_index]
                .name
                .to_string(),
            target: AnimatedPropertyTarget::from(&property.target),
        }
    }

    pub fn into(self, schema: &GeneratorSchema) -> Option<animation_clip::AnimatedProperty> {
        let group_index = match schema
            .groups
            .iter()
            .position(|group| group.name == self.group_name)
        {
            Some(group_index) => group_index,
            None => {
                eprintln!(
                    "Couldn't find group {} on schema {}, so animation will be deleted.",
                    self.group_name, schema.name
                );
                return None;
            }
        };
        let prop_index = match schema.groups[group_index]
            .properties
            .iter()
            .position(|prop| prop.name == self.property_name)
        {
            Some(prop_index) => prop_index,
            None => {
                eprintln!("Couldn't find property {} on schema {}'s group {}, so animation will be deleted.", self.property_name, schema.name, self.group_name);
                return None;
            }
        };

        Some(animation_clip::AnimatedProperty {
            group_index,
            property_index: prop_index,
            target: self.target.into(),
            is_collapsed: false,
        })
    }
}

#[derive(Serialize, Deserialize)]
enum AnimatedPropertyTarget {
    Joined(AnimatedPropertyField),
    Separate(Vec<AnimatedPropertyField>),
}

impl From<&animation_clip::AnimatedPropertyTarget> for AnimatedPropertyTarget {
    fn from(target: &animation_clip::AnimatedPropertyTarget) -> Self {
        match target {
            animation_clip::AnimatedPropertyTarget::Joined(field) => {
                AnimatedPropertyTarget::Joined(AnimatedPropertyField::from(field))
            }
            animation_clip::AnimatedPropertyTarget::Separate(fields) => {
                AnimatedPropertyTarget::Separate(
                    fields.iter().map(AnimatedPropertyField::from).collect(),
                )
            }
        }
    }
}

impl Into<animation_clip::AnimatedPropertyTarget> for AnimatedPropertyTarget {
    fn into(self) -> animation_clip::AnimatedPropertyTarget {
        match self {
            AnimatedPropertyTarget::Joined(field) => {
                animation_clip::AnimatedPropertyTarget::Joined(field.into())
            }
            AnimatedPropertyTarget::Separate(fields) => {
                animation_clip::AnimatedPropertyTarget::Separate(
                    fields.into_iter().map(|field| field.into()).collect(),
                )
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AnimatedPropertyField {
    local_offset_frames: i32,
    start_value: PropertyValue,
    segments: Vec<CurveSegment>,
}

impl From<&animation_clip::AnimatedPropertyField> for AnimatedPropertyField {
    fn from(field: &animation_clip::AnimatedPropertyField) -> Self {
        AnimatedPropertyField {
            local_offset_frames: field.local_offset_frames,
            start_value: PropertyValue::from(field.start_value),
            segments: field
                .segments
                .iter()
                .map(|segment| CurveSegment::from(segment))
                .collect(),
        }
    }
}

impl Into<animation_clip::AnimatedPropertyField> for AnimatedPropertyField {
    fn into(self) -> animation_clip::AnimatedPropertyField {
        animation_clip::AnimatedPropertyField {
            local_offset_frames: self.local_offset_frames,
            start_value: (&self.start_value).into(),
            segments: self.segments.iter().map(|segment| segment.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CurveSegment {
    duration_frames: u32,
    end_value: PropertyValue,
    interpolation: CurveInterpolation,
}

impl From<&animation_clip::CurveSegment> for CurveSegment {
    fn from(segment: &animation_clip::CurveSegment) -> Self {
        CurveSegment {
            duration_frames: segment.duration_frames,
            end_value: PropertyValue::from(segment.end_value),
            interpolation: CurveInterpolation::from(&segment.interpolation),
        }
    }
}

impl Into<animation_clip::CurveSegment> for &CurveSegment {
    fn into(self) -> animation_clip::CurveSegment {
        animation_clip::CurveSegment {
            duration_frames: self.duration_frames,
            end_value: (&self.end_value).into(),
            interpolation: (&self.interpolation).into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CurveInterpolation {
    Linear,
    CubicBezier(CubicBezier),
}

impl From<&animation_clip::CurveInterpolation> for CurveInterpolation {
    fn from(interpolation: &animation_clip::CurveInterpolation) -> Self {
        match interpolation {
            animation_clip::CurveInterpolation::Linear => CurveInterpolation::Linear,
            animation_clip::CurveInterpolation::CubicBezier(bezier) => {
                CurveInterpolation::CubicBezier(CubicBezier::from(bezier))
            }
        }
    }
}

impl Into<animation_clip::CurveInterpolation> for &CurveInterpolation {
    fn into(self) -> animation_clip::CurveInterpolation {
        match self {
            CurveInterpolation::Linear => animation_clip::CurveInterpolation::Linear,
            CurveInterpolation::CubicBezier(bezier) => {
                animation_clip::CurveInterpolation::CubicBezier(bezier.into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CubicBezier {
    c1: Point,
    c2: Point,
}

impl From<&cubic_bezier::CubicBezier> for CubicBezier {
    fn from(bezier: &cubic_bezier::CubicBezier) -> Self {
        CubicBezier {
            c1: Point {
                x: bezier.c1().x,
                y: bezier.c1().y,
            },
            c2: Point {
                x: bezier.c2().x,
                y: bezier.c2().y,
            },
        }
    }
}

impl Into<cubic_bezier::CubicBezier> for &CubicBezier {
    fn into(self) -> cubic_bezier::CubicBezier {
        cubic_bezier::CubicBezier::new(
            math::Vector2 {
                x: self.c1.x,
                y: self.c1.y,
            },
            math::Vector2 {
                x: self.c2.x,
                y: self.c2.y,
            },
        )
    }
}

#[derive(Serialize, Deserialize)]
struct Point {
    x: f32,
    y: f32,
}
