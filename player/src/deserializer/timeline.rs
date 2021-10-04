use super::Stream;
use alloc::string::String;
use alloc::vec::Vec;
use core::{intrinsics, mem};
use engine::animation::animation_clip::{
    AnimatedProperty, AnimatedPropertyField, AnimatedPropertyTarget, AnimationClip,
    CurveInterpolation, CurveSegment,
};
use engine::animation::clip::ClipReference;
use engine::animation::cubic_bezier::CubicBezier;
use engine::animation::property::{PropertyType, PropertyValue};
use engine::animation::timeline::{
    Clip, ClipSource, PropertyDefault, PropertyGroup, Timeline, Track,
};
use engine::creation_context::CreationContext;
use engine::generator::GENERATOR_SCHEMAS;

struct ClipStream<'bytes> {
    len: usize,
    start_times: Stream<'bytes>,
    durations: Stream<'bytes>,
    types: Stream<'bytes>,
}

impl<'bytes> ClipStream<'bytes> {
    fn new(mut stream: Stream<'bytes>, len: usize) -> Self {
        let start_times = stream.substream(len * mem::size_of::<u32>());
        let durations = stream.substream(len * mem::size_of::<u32>());
        let types = stream;

        ClipStream {
            len,
            start_times,
            durations,
            types,
        }
    }
}

struct AnimationClipStream<'bytes> {
    len: usize,
    targets: Stream<'bytes>,
    schemas: Stream<'bytes>,
    num_props: Stream<'bytes>,
}

impl<'bytes> AnimationClipStream<'bytes> {
    fn new(mut stream: Stream<'bytes>, len: usize) -> Self {
        let targets = stream.substream(len * mem::size_of::<u8>());
        let schemas = stream.substream(len * mem::size_of::<u8>());
        let num_props = stream;

        AnimationClipStream {
            len,
            targets,
            schemas,
            num_props,
        }
    }
}

struct AnimationPropertyStream<'bytes> {
    len: usize,
    target_groups: Stream<'bytes>,
    target_props: Stream<'bytes>,
    num_fields: Stream<'bytes>,
}

impl<'bytes> AnimationPropertyStream<'bytes> {
    fn new(mut stream: Stream<'bytes>, len: usize) -> Self {
        let target_groups = stream.substream(len * mem::size_of::<u8>());
        let target_props = stream.substream(len * mem::size_of::<u8>());
        let num_fields = stream;

        AnimationPropertyStream {
            len,
            target_groups,
            target_props,
            num_fields,
        }
    }
}

struct AnimationFieldStream<'bytes> {
    len: usize,
    local_offsets: Stream<'bytes>,
    num_segments: Stream<'bytes>,
}

impl<'bytes> AnimationFieldStream<'bytes> {
    fn new(mut stream: Stream<'bytes>, len: usize) -> Self {
        let local_offsets = stream.substream(len * mem::size_of::<i32>());
        let num_segments = stream;

        AnimationFieldStream {
            len,
            local_offsets,
            num_segments,
        }
    }
}

struct SegmentStream<'bytes> {
    len: usize,
    durations: Stream<'bytes>,
    interpolations: Stream<'bytes>,
}

impl<'bytes> SegmentStream<'bytes> {
    fn new(mut stream: Stream<'bytes>, len: usize) -> Self {
        let durations = stream.substream(len * mem::size_of::<u32>());
        let interpolations = stream;

        SegmentStream {
            len,
            durations,
            interpolations,
        }
    }
}

struct PropValStream<'bytes> {
    streams: [Stream<'bytes>; 4],
}

fn deserialize_prop_val(
    val_type: PropertyType,
    prop_val_stream: &mut PropValStream,
) -> PropertyValue {
    if val_type == PropertyType::ClipReference {
        let ref_val = prop_val_stream.streams[0].read_u8();
        let clip_ref = if ref_val == !0u8 {
            None
        } else {
            Some(ClipReference::new(ref_val as u32))
        };

        PropertyValue::ClipReference(clip_ref)
    } else {
        PropertyValue::from_fields(
            val_type,
            &mut prop_val_stream
                .streams
                .iter_mut()
                .enumerate()
                .map(|(_stream_index, stream)| stream.read_f32()),
        )
        .unwrap()
    }
}

fn deserialize_animation_field(
    val_type: PropertyType,
    prop_val_stream: &mut PropValStream,
    field_stream: &mut AnimationFieldStream,
    segment_stream: &mut SegmentStream,
) -> AnimatedPropertyField {
    let local_offset_frames = field_stream.local_offsets.read_i32();
    let start_value = deserialize_prop_val(val_type, prop_val_stream);
    let num_segments = field_stream.num_segments.read_u8();

    let mut segments = Vec::new();
    segments.reserve(num_segments as usize);
    for _ in 0..num_segments {
        let duration_frames = segment_stream.durations.read_u32();
        let end_value = deserialize_prop_val(val_type, prop_val_stream);
        let interpolation = match segment_stream.interpolations.read_u8() {
            0u8 => CurveInterpolation::Linear,
            1u8 => {
                let c1 = segment_stream.interpolations.read_vector2();
                let c2 = segment_stream.interpolations.read_vector2();
                CurveInterpolation::CubicBezier(CubicBezier::new(c1, c2))
            }
            _ => unsafe { intrinsics::unreachable() },
        };

        segments.push(CurveSegment {
            duration_frames: duration_frames * 2,
            end_value,
            interpolation,
        });
    }

    AnimatedPropertyField {
        local_offset_frames: local_offset_frames * 2,
        start_value,
        segments,
    }
}

pub fn deserialize_timeline(
    stream: &mut Stream,
    creation_context: &mut CreationContext,
    progress: &mut FnMut(f32),
) -> (u32, Timeline) {
    let project_duration = stream.read_u32();

    // Extract individual streams from the master one
    let clip_count = stream.read_u8();
    let mut clip_stream = ClipStream::new(stream.read_substream(), clip_count as usize);

    let animation_clip_count = stream.read_u8();
    let mut animation_clip_stream =
        AnimationClipStream::new(stream.read_substream(), animation_clip_count as usize);

    let animation_prop_count = stream.read_u8();
    let mut animation_prop_stream =
        AnimationPropertyStream::new(stream.read_substream(), animation_prop_count as usize);

    let animation_field_count = stream.read_u8();
    let mut animation_field_stream =
        AnimationFieldStream::new(stream.read_substream(), animation_field_count as usize);

    let segment_count = stream.read_u8();
    let mut segment_stream = SegmentStream::new(stream.read_substream(), segment_count as usize);

    let x_val_stream = stream.read_substream();
    let y_val_stream = stream.read_substream();
    let z_val_stream = stream.read_substream();
    let w_val_stream = stream.read_substream();
    let mut prop_val_stream = PropValStream {
        streams: [x_val_stream, y_val_stream, z_val_stream, w_val_stream],
    };

    // Magically transmute the streams into a timeline object
    // n.b. To simplify things here, we make one track per clip. In the future, we can probably
    // completely disregard tracks in the replayer.
    let mut tracks = Vec::new();
    tracks.reserve(clip_stream.len);
    for clip_id in 0..clip_stream.len {
        let clip_start_time = clip_stream.start_times.read_u32();
        let clip_duration = clip_stream.durations.read_u32();

        let clip_source_id = clip_stream.types.read_u8();
        let (schema, clip_source, prop_groups) = if clip_source_id == !0u8 {
            let target_clip_id = animation_clip_stream.targets.read_u8();
            let target_schema =
                &GENERATOR_SCHEMAS[animation_clip_stream.schemas.read_u8() as usize];
            let num_props = animation_clip_stream.num_props.read_u8();

            let mut animated_properties = Vec::new();
            animated_properties.reserve(num_props as usize);
            for _ in 0..num_props {
                let target_group = animation_prop_stream.target_groups.read_u8();
                let target_prop = animation_prop_stream.target_props.read_u8();
                let target_type = target_schema.groups[target_group as usize].properties
                    [target_prop as usize]
                    .value_type;
                let num_fields = animation_prop_stream.num_fields.read_u8();

                let target = if num_fields == 0 {
                    let field = deserialize_animation_field(
                        target_type,
                        &mut prop_val_stream,
                        &mut animation_field_stream,
                        &mut segment_stream,
                    );
                    AnimatedPropertyTarget::Joined(field)
                } else {
                    let mut fields = Vec::new();
                    fields.reserve(num_fields as usize);
                    for _ in 0..num_fields {
                        let field = deserialize_animation_field(
                            PropertyType::Float,
                            &mut prop_val_stream,
                            &mut animation_field_stream,
                            &mut segment_stream,
                        );
                        fields.push(field);
                    }
                    AnimatedPropertyTarget::Separate(fields)
                };

                animated_properties.push(AnimatedProperty {
                    group_index: target_group as usize,
                    property_index: target_prop as usize,
                    target,
                    is_collapsed: false,
                });
            }

            let clip_source = ClipSource::Animation(AnimationClip {
                target_clip: ClipReference::new(target_clip_id as u32),
                properties: animated_properties,
            });
            (target_schema, clip_source, Vec::new())
        } else {
            let schema = &GENERATOR_SCHEMAS[clip_source_id as usize];
            let prop_groups: Vec<_> = schema
                .groups
                .iter()
                .map(|schema_group| {
                    let prop_defaults: Vec<_> = schema_group
                        .properties
                        .iter()
                        .map(|schema_prop| PropertyDefault {
                            value: deserialize_prop_val(
                                schema_prop.value_type,
                                &mut prop_val_stream,
                            ),
                            is_override: false,
                        })
                        .collect();
                    PropertyGroup {
                        defaults: prop_defaults,
                    }
                })
                .collect();

            let clip_source =
                ClipSource::Generator((schema.instantiate_generator)(creation_context));
            (schema, clip_source, prop_groups)
        };

        progress((clip_id + 1) as f32 / clip_stream.len as f32);

        let clip = Clip {
            #[cfg(debug_assertions)]
            name: String::new(),

            id: clip_id as u32,
            schema,
            source: clip_source,

            offset_frames: clip_start_time * 2,
            duration_frames: clip_duration * 2,

            property_groups: prop_groups,
            is_selected: false,
        };
        tracks.push(Track { clips: vec![clip] });
    }

    (project_duration, Timeline { tracks })
}
