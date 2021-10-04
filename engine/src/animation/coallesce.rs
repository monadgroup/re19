use super::animation_clip::{AnimatedPropertyField, AnimatedPropertyTarget};
use super::clip::ActiveClipMap;
use super::property::PropertyValue;
use super::timeline::{ClipSource, Timeline};

pub fn coallesce_animations(timeline: &Timeline, clip_map: &mut ActiveClipMap) {
    for active_clip_index in 0..clip_map.active_clips_mut().len() {
        let active_clip = &clip_map.active_clips_mut()[active_clip_index];
        let active_local_time = active_clip.local_time;
        let active_reference = active_clip.reference;
        let clip = match timeline.tracks[active_clip.track_index]
            .clips
            .get(active_clip.clip_index)
        {
            Some(clip) => clip,
            None => continue, // the clip has been deleted
        };

        // If the clip is an animation, process it and apply it to the relevant buffer
        let animation_clip = match &clip.source {
            ClipSource::Animation(animation) => animation,
            _ => continue,
        };
        let target_clip = match clip_map.get_clip_index(animation_clip.target_clip) {
            Some(index) => &mut clip_map.active_clips_mut()[index],
            None => continue,
        };

        // Evaluate the clip's time property to use when calculating the actual fields
        //let time_progress =
        //    get_animation_field_value(&animation_clip.time_property, active_local_time as f32)
        //        .into_float()
        //        .unwrap();
        //let progress_local_time = clip.duration_frames as f32 * time_progress;
        let progress_local_time = active_local_time as f32;

        // Apply each animation property
        for animated_property in &animation_clip.properties {
            let target_property_val = &mut target_clip.properties[animated_property.group_index]
                [animated_property.property_index];

            target_property_val.targeted_by = Some(active_reference);

            // skip the property if it's overridden
            if target_property_val.is_overridden {
                continue;
            }

            let animated_value = match &animated_property.target {
                AnimatedPropertyTarget::Joined(field) => {
                    get_animation_field_value(field, progress_local_time)
                }
                AnimatedPropertyTarget::Separate(fields) => {
                    let mut value_iter = fields.iter().map(|field| {
                        get_animation_field_value(field, progress_local_time)
                            .into_float()
                            .unwrap()
                    });
                    let target_type = target_property_val.value.get_type();
                    PropertyValue::from_fields(target_type, &mut value_iter).unwrap()
                }
            };

            target_property_val.value = animated_value;
        }
    }
}

fn get_animation_field_value(field: &AnimatedPropertyField, local_clip_time: f32) -> PropertyValue {
    let local_field_time = local_clip_time - field.local_offset_frames as f32;
    if local_field_time < 0. {
        return field.start_value;
    }

    // Find the segment that's active at the current time, as well as the start time of the last one
    let mut last_end_val = field.start_value;
    let mut last_end_time = 0;
    for segment in &field.segments {
        if local_field_time < (last_end_time + segment.duration_frames) as f32 {
            // We found a clip! Apply interpolation on it.
            let local_curve_time = local_field_time - last_end_time as f32;
            let curve_progress = local_curve_time / segment.duration_frames as f32;
            let curve_lerp = segment.interpolation.eval(curve_progress);

            return last_end_val.lerp(segment.end_value, curve_lerp).unwrap();
        }

        last_end_val = segment.end_value;
        last_end_time += segment.duration_frames;
    }

    // We didn't find a matching segment, so we must be after the curve - just return the value of
    // the last one.
    last_end_val
}
