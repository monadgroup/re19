use crate::editor_state::EditorState;
use engine::animation::animation_clip::{
    AnimatedPropertyField, AnimatedPropertyTarget, CurveInterpolation, CurveSegment,
};
use engine::animation::property::PropertyValue;
use engine::animation::timeline::{Clip, ClipSource, Timeline, Track};
use std::{iter, slice};

pub fn deselect_all_clips(timeline: &mut Timeline) {
    for track in &mut timeline.tracks {
        for clip in &mut track.clips {
            clip.is_selected = false;
        }
    }
}

pub fn select_clip(
    timeline: &mut Timeline,
    track_index: usize,
    clip_index: usize,
    exclusive: bool,
) {
    if exclusive {
        // if the clip is already selected, don't do anything
        if timeline.tracks[track_index].clips[clip_index].is_selected {
            return;
        }

        deselect_all_clips(timeline);
    }

    let clip = &mut timeline.tracks[track_index].clips[clip_index];
    clip.is_selected = !clip.is_selected;
}
pub fn can_fit_clip(
    track: &Track,
    place_position_frames: u32,
    place_duration_frames: u32,
    ignore_selected: bool,
) -> bool {
    let start_frames = place_position_frames;
    let end_frames = place_position_frames + place_duration_frames;

    // Associate each clip with its start time and filter out the ones that don't intersect
    track
        .clips
        .iter()
        .scan(0, |last_clip_end, clip| {
            let clip_start_time = *last_clip_end + clip.offset_frames;
            let clip_end_time = clip_start_time + clip.duration_frames;
            *last_clip_end = clip_end_time;

            Some((
                clip_start_time,
                clip_end_time,
                !ignore_selected || !clip.is_selected,
            ))
        })
        .filter(|&(_, _, pred)| pred)
        .all(|(clip_start_time, clip_end_time, _)| {
            (clip_start_time < start_frames || clip_start_time >= end_frames)
                && (start_frames < clip_start_time || start_frames >= clip_end_time)
        })
}

pub fn remove_clip(track: &mut Track, index: usize) -> Clip {
    let removed_clip = track.clips.remove(index);
    if let Some(after_clip) = track.clips.get_mut(index) {
        after_clip.offset_frames += removed_clip.offset_frames + removed_clip.duration_frames;
    }
    removed_clip
}

pub fn insert_clip(track: &mut Track, mut clip: Clip, position_frames: u32) -> Result<(), Clip> {
    // Find the index of the clip that will come before this one
    let before_index = track
        .clips
        .iter()
        .enumerate()
        .scan(0, |last_clip_end, (clip_index, clip)| {
            let clip_start_time = *last_clip_end + clip.offset_frames;
            let clip_end_time = clip_start_time + clip.duration_frames;
            *last_clip_end = clip_end_time;

            Some((clip_start_time, clip_end_time, clip_index))
        })
        .take_while(|&(clip_start_time, _, _)| clip_start_time < position_frames)
        .last();

    // Find the index of the clip that will come after us
    let after_index = match before_index {
        Some((_, _, index)) => index + 1,
        None => 0,
    };

    let before_end_time = match before_index {
        Some((_, end_time, _)) => end_time,
        None => 0,
    };

    let clip_offset_frames = position_frames - before_end_time;

    // Update the offset of the next clip if there is one (and make sure there's actually space)
    if let Some(after_clip) = track.clips.get_mut(after_index) {
        if after_clip.offset_frames < clip_offset_frames + clip.duration_frames {
            // Not enough space to fit the clip, abort!
            return Err(clip);
        }

        after_clip.offset_frames -= clip_offset_frames + clip.duration_frames;
    }

    clip.offset_frames = clip_offset_frames;
    track.clips.insert(after_index, clip);

    Ok(())
}

pub fn insert_keyframe(
    field: &mut AnimatedPropertyField,
    position_frames: i32,
    value: PropertyValue,
) -> bool {
    // Special case: if the position is before the start time, do a swap operation
    if position_frames < field.local_offset_frames {
        let new_curve_duration = (field.local_offset_frames - position_frames) as u32;
        field.segments.insert(
            0,
            CurveSegment {
                duration_frames: new_curve_duration,
                end_value: field.start_value,
                interpolation: CurveInterpolation::Linear,
            },
        );
        field.local_offset_frames = position_frames;
        field.start_value = value;
        return true;
    }

    // Special case: if the position is at the start time, update the start value
    if position_frames == field.local_offset_frames {
        field.start_value = value;
        return true;
    }

    // Find the index of the segment we're inside
    let segments = field.segments.iter().enumerate().scan(
        field.local_offset_frames,
        |last_segment_end, (segment_index, segment)| {
            let segment_start_time = *last_segment_end;
            let segment_end_time = segment_start_time + segment.duration_frames as i32;
            *last_segment_end = segment_end_time;

            Some((segment_end_time, segment_index))
        },
    );
    let mut segment_index = None;
    for (segment_end_time, segment_idx) in segments {
        if position_frames <= segment_end_time {
            segment_index = Some((segment_end_time, segment_idx));
            break;
        }
    }

    if let Some((segment_end_time, segment_index)) = segment_index {
        let segment = &mut field.segments[segment_index];

        // If we're sitting on the end of the segment, just update the segment's value
        if position_frames == segment_end_time {
            segment.end_value = value;
            return true;
        }

        let segment_start_time = segment_end_time - segment.duration_frames as i32;
        let new_segment_duration = (position_frames - segment_start_time) as u32;
        let old_segment_duration = segment.duration_frames - new_segment_duration;
        segment.duration_frames = old_segment_duration;

        // Insert the new segment as needed
        field.segments.insert(
            segment_index,
            CurveSegment {
                duration_frames: new_segment_duration,
                end_value: value,
                interpolation: CurveInterpolation::Linear,
            },
        );

        true
    } else {
        // We're inserting after all of the segments
        let last_segment_end = field
            .segments
            .iter()
            .fold(field.local_offset_frames, |last_segment_end, segment| {
                last_segment_end + segment.duration_frames as i32
            });
        field.segments.push(CurveSegment {
            duration_frames: (position_frames - last_segment_end) as u32,
            end_value: value,
            interpolation: CurveInterpolation::Linear,
        });

        true
    }
}

pub fn delete_keyframe(field: &mut AnimatedPropertyField, index: Option<usize>) {
    match index {
        None => {
            // This is the first keyframe - prevent deleting if there aren't any other ones
            let first_segment = match field.segments.first() {
                Some(first_segment) => first_segment,
                None => return,
            };

            field.start_value = first_segment.end_value;
            field.local_offset_frames += first_segment.duration_frames as i32;
            field.segments.remove(0);
        }
        Some(index) => {
            let removed_segment = field.segments.remove(index);
            if let Some(next_segment) = field.segments.get_mut(index) {
                next_segment.duration_frames += removed_segment.duration_frames;
            }
        }
    }
}

pub fn move_selected_clips(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    frame_offset: i32,
) {
    let real_offset = frame_offset - editor_state.drag_offset;
    if real_offset == 0 {
        return;
    }

    // Make sure all selected clips can be moved by the provided offset
    let can_move_clips = timeline
        .tracks
        .iter()
        .flat_map(|track| {
            track.clips.iter().scan(0, move |last_clip_end, clip| {
                let clip_start_time = *last_clip_end + clip.offset_frames;
                *last_clip_end = clip_start_time + clip.duration_frames;

                Some((track, clip, clip_start_time))
            })
        })
        .filter(|&(_, clip, _)| clip.is_selected)
        .all(|(track, clip, clip_start_time)| {
            // If the offset will move the clip to start before 0, we can't do it
            if real_offset < 0 && (-real_offset) as u32 > clip_start_time {
                return false;
            }

            can_fit_clip(
                track,
                (clip_start_time as i32 + real_offset) as u32,
                clip.duration_frames,
                true,
            )
        });

    if !can_move_clips {
        return;
    }

    for track in &mut timeline.tracks {
        // First, remove all clips that are selected, while recording their new start times
        let mut move_clips = Vec::new();
        let mut last_clip_end = 0;
        let mut index = 0;
        while index < track.clips.len() {
            let clip = &track.clips[index];
            let clip_start_time = last_clip_end + clip.offset_frames;

            if clip.is_selected {
                let new_start_time = (clip_start_time as i32 + real_offset) as u32;
                move_clips.push((new_start_time, remove_clip(track, index)));
            } else {
                index += 1;
                last_clip_end = clip_start_time + clip.duration_frames;
            }
        }

        // Now re-insert the clips into the track
        for (new_start_time, clip) in move_clips.into_iter() {
            insert_clip(track, clip, new_start_time).ok().unwrap();
        }
    }

    editor_state.drag_offset = frame_offset;
}

struct GenerateIter<I, F>
where
    F: FnMut() -> I,
{
    remaining: usize,
    generator: F,
}

impl<I, F> GenerateIter<I, F>
where
    F: FnMut() -> I,
{
    pub fn new(length: usize, generator: F) -> Self {
        GenerateIter {
            remaining: length,
            generator,
        }
    }
}

impl<I, F> Iterator for GenerateIter<I, F>
where
    F: FnMut() -> I,
{
    type Item = I;

    fn next(&mut self) -> Option<I> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            Some((self.generator)())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

pub fn change_selected_clip_tracks(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    track_offset: i32,
) {
    let real_offset = track_offset - editor_state.track_offset;
    if real_offset == 0 {
        return;
    }

    // Make sure all the selected clips can be moved to the specified track
    let can_move_clips = timeline
        .tracks
        .iter()
        .enumerate()
        .flat_map(|(track_index, track)| {
            track
                .clips
                .iter()
                .scan(0, |last_clip_end, clip| {
                    let clip_start_time = *last_clip_end + clip.offset_frames;
                    *last_clip_end = clip_start_time + clip.duration_frames;

                    Some((clip_start_time, clip))
                })
                .map(move |(clip_start_time, clip)| (track_index, clip_start_time, clip))
        })
        .filter(|&(_, _, clip)| clip.is_selected)
        .all(|(track_index, clip_start_time, clip)| {
            let new_track_index = track_index as i32 + real_offset;

            // If moving into a new track, no need to check
            if new_track_index < 0 || new_track_index >= timeline.tracks.len() as i32 {
                return true;
            }

            can_fit_clip(
                &timeline.tracks[new_track_index as usize],
                clip_start_time,
                clip.duration_frames,
                true,
            )
        });

    if !can_move_clips {
        return;
    }

    // Determine the min/max track indices, so we can create new ones if required
    let (min_track_index, max_track_index) = timeline
        .tracks
        .iter()
        .enumerate()
        .filter(|&(_, track)| track.clips.iter().any(|clip| clip.is_selected))
        .fold(
            (0, 0),
            |(min_track_index, max_track_index), (track_index, _)| {
                let new_track_index = track_index as i32 + real_offset;

                (
                    min_track_index.min(new_track_index),
                    max_track_index.max(new_track_index),
                )
            },
        );

    let track_index_offset = if min_track_index < 0 {
        -min_track_index as usize
    } else {
        0
    };

    // Pull all selected items out of the timeline
    let mut pulled_clips = Vec::new();
    for (track_index, track) in timeline.tracks.iter_mut().enumerate() {
        let new_track_index = ((track_index + track_index_offset) as i32 + real_offset) as usize;

        let mut last_clip_end = 0;
        let mut clip_index = 0;
        while clip_index < track.clips.len() {
            let clip = &track.clips[clip_index];
            let clip_start_time = last_clip_end + clip.offset_frames;

            if clip.is_selected {
                pulled_clips.push((
                    new_track_index,
                    clip_start_time,
                    remove_clip(track, clip_index),
                ));
            } else {
                clip_index += 1;
                last_clip_end = clip_start_time + clip.duration_frames;
            }
        }
    }

    // Append new tracks to the end as necessary
    if max_track_index >= timeline.tracks.len() as i32 {
        timeline
            .tracks
            .resize_with(max_track_index as usize + 1, Track::default);
    }

    // Add new tracks to the start if necessary
    if track_index_offset > 0 {
        timeline
            .tracks
            .splice(0..0, GenerateIter::new(track_index_offset, Track::default));
    }

    // Put the items back in their new positions
    for (target_track, clip_position, clip) in pulled_clips.into_iter() {
        insert_clip(&mut timeline.tracks[target_track], clip, clip_position)
            .ok()
            .unwrap();
    }

    trim_empty_tracks(timeline);

    editor_state.track_offset = track_offset;
}

pub fn trim_empty_tracks(timeline: &mut Timeline) {
    let start_empty_tracks = timeline
        .tracks
        .iter()
        .take_while(|track| track.clips.is_empty())
        .count();
    timeline.tracks.splice(0..start_empty_tracks, iter::empty());

    let end_empty_tracks = timeline
        .tracks
        .iter()
        .rev()
        .take_while(|track| track.clips.is_empty())
        .count();
    timeline.tracks.splice(
        (timeline.tracks.len() - end_empty_tracks)..timeline.tracks.len(),
        iter::empty(),
    );

    if timeline.tracks.is_empty() {
        timeline.tracks.push(Track::default());
    }
}

pub fn resize_selected_clips_left(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    frame_offset: i32,
    min_duration: u32,
) {
    let real_offset = frame_offset - editor_state.drag_offset;

    // Make sure we have space to resize the clips
    let can_resize = timeline
        .tracks
        .iter()
        .flat_map(|track| track.clips.iter())
        .filter(|clip| clip.is_selected)
        .all(|clip| {
            if real_offset > 0 {
                // Resizing to make the clip smaller
                (real_offset as u32) + min_duration <= clip.duration_frames
            } else {
                // Resizing to make the clip bigger
                -real_offset as u32 <= clip.offset_frames
            }
        });

    if !can_resize {
        return;
    }

    for track in &mut timeline.tracks {
        for clip in &mut track.clips {
            if !clip.is_selected {
                continue;
            }

            if real_offset > 0 {
                clip.duration_frames -= real_offset as u32;
                clip.offset_frames += real_offset as u32;
            } else {
                clip.duration_frames += -real_offset as u32;
                clip.offset_frames -= -real_offset as u32;
            }

            // For animation clips, adjust the starting time for all fields _EXCEPT_ the time one
            // (because that would mess things up pretty badly!)
            // review: maybe we do want to do something with the time field though?
            if let ClipSource::Animation(animation) = &mut clip.source {
                let animation_fields =
                    animation
                        .properties
                        .iter_mut()
                        .flat_map(|prop| match &mut prop.target {
                            AnimatedPropertyTarget::Joined(field) => slice::from_mut(field),
                            AnimatedPropertyTarget::Separate(fields) => fields,
                        });
                for animation_field in animation_fields {
                    // todo: is this the correct direction?
                    animation_field.local_offset_frames -= real_offset;
                }
            }
        }
    }

    editor_state.drag_offset = frame_offset;
}

pub fn resize_selected_clips_right(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    frame_offset: i32,
    min_duration: u32,
) {
    let real_offset = frame_offset - editor_state.drag_offset;

    // Make sure we have space to resize the clips
    let can_resize = timeline
        .tracks
        .iter()
        .flat_map(|track| {
            track
                .clips
                .iter()
                .rev()
                .scan(None, |next_clip_ref, this_clip| {
                    let next_clip = *next_clip_ref;
                    *next_clip_ref = Some(this_clip);

                    Some((this_clip, next_clip))
                })
        })
        .filter(|&(this_clip, _)| this_clip.is_selected)
        .all(|(this_clip, next_clip)| {
            if real_offset > 0 {
                // Resizing to make the clip bigger
                if let Some(next_clip) = next_clip {
                    real_offset as u32 <= next_clip.offset_frames
                } else {
                    true
                }
            } else {
                // Resizing to make the clip smaller
                (-real_offset as u32) + min_duration <= this_clip.duration_frames
            }
        });

    if !can_resize {
        return;
    }

    for track in &mut timeline.tracks {
        for clip_index in 0..track.clips.len() {
            let clip = &mut track.clips[clip_index];
            if !clip.is_selected {
                continue;
            }

            if real_offset > 0 {
                clip.duration_frames += real_offset as u32;
            } else {
                clip.duration_frames -= -real_offset as u32;
            }

            // Scale the time property as needed
            /*if let ClipSource::Animation(ref mut animation_clip) = clip.source {
                let scale_factor = clip.duration_frames as f32 / original_duration as f32;
                animation_clip.time_property.local_offset_frames = (animation_clip.time_property.local_offset_frames as f32 * scale_factor) as i32;
                for segment in &mut animation_clip.time_property.segments {
                    segment.duration_frames = (segment.duration_frames as f32 * scale_factor) as u32;
                }
            }*/

            if let Some(next_clip) = track.clips.get_mut(clip_index + 1) {
                if real_offset > 0 {
                    next_clip.offset_frames -= real_offset as u32;
                } else {
                    next_clip.offset_frames += -real_offset as u32;
                }
            }
        }
    }

    editor_state.drag_offset = frame_offset;
}

/// Finds all snapping points for dragging. Essentially, this means that if the mouse is near one
/// of these points relative to it's start position, it should snap to the next smallest one(?)
/// Each (non-selected) clip generates two snapping targets, one at the start and one at the end.
/// Each selected clip has two snapping sources, also one at each end.
/// The result is a permutation of the two, by finding the required movement of the mouse to make
/// each source reach each target.
pub fn get_snapping_points(timeline: &Timeline) -> Vec<i32> {
    let all_clips: Vec<_> = timeline
        .tracks
        .iter()
        .flat_map(|track| {
            track.clips.iter().scan(0, |last_clip_end, clip| {
                let clip_start_time = *last_clip_end + clip.offset_frames;
                *last_clip_end = clip_start_time + clip.duration_frames;

                Some((clip_start_time, clip))
            })
        })
        .collect();

    let snap_targets: Vec<_> = all_clips
        .iter()
        .filter(|&&(_, clip)| !clip.is_selected)
        .flat_map(|&(clip_start_time, clip)| {
            iter::once(clip_start_time).chain(iter::once(clip_start_time + clip.duration_frames))
        })
        .chain(iter::once(0))
        .collect();

    let mut snap_offsets: Vec<_> = all_clips
        .iter()
        .filter(|&&(_, clip)| clip.is_selected)
        .flat_map(|&(clip_start_time, clip)| {
            iter::once(clip_start_time).chain(iter::once(clip_start_time + clip.duration_frames))
        })
        .flat_map(|snap_source_frame| {
            snap_targets.iter().map(move |&snap_target_frame| {
                (snap_target_frame as i32) - (snap_source_frame as i32)
            })
        })
        .collect();

    snap_offsets.sort_unstable();
    snap_offsets.dedup();

    snap_offsets
}

pub fn snap_offset(offset: i32, snapping_points: &[i32], threshold: i32) -> i32 {
    // no point trying to snap if the threshold is 0
    if threshold <= 0 {
        return offset;
    }

    snapping_points
        .iter()
        .fold(
            (offset, threshold),
            |(closest_snapped, closest_distance), &snapping_point| {
                let current_distance = (snapping_point - offset).abs();
                if current_distance < closest_distance {
                    (snapping_point, current_distance)
                } else {
                    (closest_snapped, closest_distance)
                }
            },
        )
        .0
}
