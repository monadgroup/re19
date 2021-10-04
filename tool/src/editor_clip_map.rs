use engine::animation::clip::{ActiveClip, ActiveClipMap, ClipPropertyValue, ClipReference};
use engine::animation::timeline::Timeline;
use std::collections::HashMap;
use std::iter::FromIterator;

pub struct EditorClipMap {
    pub active_clips: Vec<ActiveClip>,
    pub map: HashMap<ClipReference, usize>,
}

impl EditorClipMap {
    pub fn from_timeline(timeline: &Timeline, current_frame: u32) -> Self {
        // Build a list of clips that are currently active
        let active_clips: Vec<_> = timeline
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(track_index, track)| {
                track
                    .clips
                    .iter()
                    .enumerate()
                    .scan(0, |last_end_time, (clip_index, clip)| {
                        let clip_start_time = *last_end_time + clip.offset_frames;
                        *last_end_time = clip_start_time + clip.duration_frames;

                        Some((clip_start_time, clip_index, clip))
                    })
                    .filter(|&(clip_start_time, _, clip)| {
                        current_frame >= clip_start_time
                            && current_frame < clip_start_time + clip.duration_frames
                    })
                    .map(|(clip_start_time, clip_index, clip)| {
                        let properties = clip
                            .property_groups
                            .iter()
                            .map(|group| {
                                group
                                    .defaults
                                    .iter()
                                    .map(|default| ClipPropertyValue {
                                        value: default.value,
                                        is_overridden: default.is_override,
                                        targeted_by: None,
                                    })
                                    .collect()
                            })
                            .collect();

                        ActiveClip {
                            name: clip.name.clone(),
                            reference: ClipReference::new(clip.id),
                            track_index,
                            clip_index,
                            local_time: current_frame - clip_start_time,
                            properties,
                        }
                    })
                    .next()
            })
            .collect();
        let map = HashMap::from_iter(
            active_clips
                .iter()
                .enumerate()
                .map(|(clip_index, active_clip)| (active_clip.reference, clip_index)),
        );

        EditorClipMap { active_clips, map }
    }
}

impl ActiveClipMap for EditorClipMap {
    fn active_clips(&self) -> &[ActiveClip] {
        &self.active_clips
    }

    fn active_clips_mut(&mut self) -> &mut [ActiveClip] {
        &mut self.active_clips
    }

    fn get_clip_index(&self, reference: ClipReference) -> Option<usize> {
        self.map.get(&reference).cloned()
    }
}
