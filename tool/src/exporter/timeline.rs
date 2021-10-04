use super::binary_writer::write;
use engine::animation::animation_clip::{
    AnimatedPropertyField, AnimatedPropertyTarget, CurveInterpolation,
};
use engine::animation::property::PropertyValue;
use engine::animation::timeline::{ClipSource, Timeline};
use engine::generator::GENERATOR_SCHEMAS;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Default)]
struct ClipStream {
    len: usize,
    start_times: Vec<u8>,
    durations: Vec<u8>,
    types: Vec<u8>,
}

#[derive(Default)]
struct AnimationClipStream {
    len: usize,
    targets: Vec<u8>,
    schemas: Vec<u8>,
    num_props: Vec<u8>,
}

#[derive(Default)]
struct AnimationPropertyStream {
    len: usize,
    target_groups: Vec<u8>,
    target_props: Vec<u8>,
    num_fields: Vec<u8>,
}

#[derive(Default)]
struct AnimationFieldStream {
    len: usize,
    local_offsets: Vec<u8>,
    num_segments: Vec<u8>,
}

#[derive(Default)]
struct SegmentStream {
    len: usize,
    durations: Vec<u8>,
    interpolations: Vec<u8>,
}

#[derive(Default)]
struct PropValStream {
    streams: [Vec<u8>; 4],
}

fn export_property_value(
    value: PropertyValue,
    id_map: &HashMap<u32, (u32, usize, usize)>,
    stream: &mut PropValStream,
) {
    if let PropertyValue::ClipReference(reference) = value {
        match reference {
            Some(ref_val) => {
                let (remapped_ref, _, _) = id_map[&ref_val.clip_id()];
                write(&mut stream.streams[0], remapped_ref as u8);
            }
            None => {
                write(&mut stream.streams[0], !0u8);
            }
        }
    } else {
        for (field_index, field) in value.fields().enumerate() {
            write(&mut stream.streams[field_index], field);
        }
    }
}

fn export_animated_field(
    field: &AnimatedPropertyField,
    id_map: &HashMap<u32, (u32, usize, usize)>,
    prop_val_stream: &mut PropValStream,
    field_stream: &mut AnimationFieldStream,
    segment_stream: &mut SegmentStream,
) {
    field_stream.len += 1;
    write(&mut field_stream.local_offsets, field.local_offset_frames);
    export_property_value(field.start_value, id_map, prop_val_stream);
    write(&mut field_stream.num_segments, field.segments.len() as u8);

    for segment in &field.segments {
        segment_stream.len += 1;
        write(&mut segment_stream.durations, segment.duration_frames);
        export_property_value(segment.end_value, id_map, prop_val_stream);
        match &segment.interpolation {
            CurveInterpolation::Linear => write(&mut segment_stream.interpolations, 0u8),
            CurveInterpolation::CubicBezier(bezier) => {
                write(&mut segment_stream.interpolations, 1u8);
                write(&mut segment_stream.interpolations, bezier.c1());
                write(&mut segment_stream.interpolations, bezier.c2());
            }
        }
    }
}

pub fn export_timeline(timeline: &Timeline, buffer: &mut Vec<u8>) {
    let clip_refs: Vec<_> = timeline
        .tracks
        .iter()
        .enumerate()
        .flat_map(|(track_index, track)| {
            track
                .clips
                .iter()
                .enumerate()
                .scan(0, move |last_end_time, (clip_index, clip)| {
                    let clip_start_time = *last_end_time + clip.offset_frames;
                    *last_end_time = clip_start_time + clip.duration_frames;

                    Some((
                        track_index,
                        clip_index,
                        clip_start_time,
                        clip_start_time + clip.duration_frames,
                    ))
                })
        })
        .collect();
    let id_map = HashMap::from_iter(clip_refs.iter().enumerate().map(
        |(new_clip_id, (track_index, clip_index, _, _))| {
            let source_clip_id = timeline.tracks[*track_index].clips[*clip_index].id;
            (
                source_clip_id,
                (new_clip_id as u32, *track_index, *clip_index),
            )
        },
    ));

    let mut clip_stream = ClipStream::default();
    let mut animation_clip_stream = AnimationClipStream::default();
    let mut animation_prop_stream = AnimationPropertyStream::default();
    let mut animation_field_stream = AnimationFieldStream::default();
    let mut segment_stream = SegmentStream::default();
    let mut prop_val_stream = PropValStream::default();

    let project_duration = clip_refs
        .iter()
        .map(|(_, _, _, clip_end_time)| clip_end_time)
        .max()
        .cloned()
        .unwrap_or(0);

    for (track_index, clip_index, clip_start_time, _) in &clip_refs {
        let clip = &timeline.tracks[*track_index].clips[*clip_index];

        clip_stream.len += 1;
        write(&mut clip_stream.start_times, *clip_start_time);
        write(&mut clip_stream.durations, clip.duration_frames);

        let schema_index = GENERATOR_SCHEMAS
            .iter()
            .position(|schema| schema.name == clip.schema.name)
            .unwrap();

        match &clip.source {
            ClipSource::Generator(_) => {
                // clip type = index of the schema in the schema list
                write(&mut clip_stream.types, schema_index as u8);

                // write each of the field values to the field stream
                for group in &clip.property_groups {
                    for default in &group.defaults {
                        export_property_value(default.value, &id_map, &mut prop_val_stream);
                    }
                }
            }
            ClipSource::Animation(animation_clip) => {
                // clip type = !0
                write(&mut clip_stream.types, !0u8);

                let (remapped_ref, target_track_index, target_clip_index) =
                    id_map[&animation_clip.target_clip.clip_id()];
                animation_clip_stream.len += 1;
                write(&mut animation_clip_stream.targets, remapped_ref as u8);
                write(&mut animation_clip_stream.schemas, schema_index as u8);

                // Build a list of properties that aren't overridden by the target clip
                let target_clip = &timeline.tracks[target_track_index].clips[target_clip_index];
                let active_animated_properties: Vec<_> = animation_clip
                    .properties
                    .iter()
                    .filter(|&prop| {
                        let targeted_default = &target_clip.property_groups[prop.group_index]
                            .defaults[prop.property_index];
                        !targeted_default.is_override
                    })
                    .collect();

                write(
                    &mut animation_clip_stream.num_props,
                    active_animated_properties.len() as u8,
                );

                // todo: skip this clip entirely if there aren't any active animated properties

                /*export_animated_field(
                    &animation_clip.time_property,
                    &id_map,
                    &mut prop_val_stream,
                    &mut animation_field_stream,
                    &mut segment_stream,
                );*/

                for &animated_property in &active_animated_properties {
                    animation_prop_stream.len += 1;
                    write(
                        &mut animation_prop_stream.target_groups,
                        animated_property.group_index as u8,
                    );
                    write(
                        &mut animation_prop_stream.target_props,
                        animated_property.property_index as u8,
                    );

                    match &animated_property.target {
                        AnimatedPropertyTarget::Joined(field) => {
                            write(&mut animation_prop_stream.num_fields, 0u8);
                            export_animated_field(
                                field,
                                &id_map,
                                &mut prop_val_stream,
                                &mut animation_field_stream,
                                &mut segment_stream,
                            );
                        }
                        AnimatedPropertyTarget::Separate(fields) => {
                            write(&mut animation_prop_stream.num_fields, fields.len() as u8);
                            for field in fields {
                                export_animated_field(
                                    field,
                                    &id_map,
                                    &mut prop_val_stream,
                                    &mut animation_field_stream,
                                    &mut segment_stream,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Composite everything into the output buffer
    write(buffer, project_duration as u32);

    write(buffer, clip_refs.len() as u8);
    write(
        buffer,
        (clip_stream.start_times.len() + clip_stream.durations.len() + clip_stream.types.len())
            as u32,
    );
    buffer.extend_from_slice(&clip_stream.start_times);
    buffer.extend_from_slice(&clip_stream.durations);
    buffer.extend_from_slice(&clip_stream.types);

    write(buffer, animation_clip_stream.len as u8);
    write(
        buffer,
        (animation_clip_stream.targets.len()
            + animation_clip_stream.schemas.len()
            + animation_clip_stream.num_props.len()) as u32,
    );
    buffer.extend_from_slice(&animation_clip_stream.targets);
    buffer.extend_from_slice(&animation_clip_stream.schemas);
    buffer.extend_from_slice(&animation_clip_stream.num_props);

    write(buffer, animation_prop_stream.len as u8);
    write(
        buffer,
        (animation_prop_stream.target_groups.len()
            + animation_prop_stream.target_props.len()
            + animation_prop_stream.num_fields.len()) as u32,
    );
    buffer.extend_from_slice(&animation_prop_stream.target_groups);
    buffer.extend_from_slice(&animation_prop_stream.target_props);
    buffer.extend_from_slice(&animation_prop_stream.num_fields);

    write(buffer, animation_field_stream.len as u8);
    write(
        buffer,
        (animation_field_stream.local_offsets.len() + animation_field_stream.num_segments.len())
            as u32,
    );
    buffer.extend_from_slice(&animation_field_stream.local_offsets);
    buffer.extend_from_slice(&animation_field_stream.num_segments);

    write(buffer, segment_stream.len as u8);
    write(
        buffer,
        (segment_stream.durations.len() + segment_stream.interpolations.len()) as u32,
    );
    buffer.extend_from_slice(&segment_stream.durations);
    buffer.extend_from_slice(&segment_stream.interpolations);

    write(buffer, prop_val_stream.streams[0].len() as u32);
    buffer.extend_from_slice(&prop_val_stream.streams[0]);

    write(buffer, prop_val_stream.streams[1].len() as u32);
    buffer.extend_from_slice(&prop_val_stream.streams[1]);

    write(buffer, prop_val_stream.streams[2].len() as u32);
    buffer.extend_from_slice(&prop_val_stream.streams[2]);

    write(buffer, prop_val_stream.streams[3].len() as u32);
    buffer.extend_from_slice(&prop_val_stream.streams[3]);
}
