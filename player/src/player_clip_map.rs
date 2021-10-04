use alloc::string::String;
use alloc::vec::Vec;
use engine::animation::clip::{ActiveClip, ActiveClipMap, ClipPropertyValue, ClipReference};
use engine::animation::timeline::Timeline;

pub struct PlayerClipMap {
    active_clips: Vec<ActiveClip>,
    clip_indices: Vec<Option<usize>>,
}

impl PlayerClipMap {
    pub fn new(clip_count: usize) -> Self {
        let mut active_clips = Vec::new();
        active_clips.reserve(clip_count);

        let mut clip_indices = Vec::new();
        clip_indices.reserve(clip_count);

        PlayerClipMap {
            active_clips,
            clip_indices,
        }
    }

    pub fn update(&mut self, timeline: &Timeline, current_frame: u32) {
        self.active_clips.clear();
        self.clip_indices.clear();

        for (track_index, track) in timeline.tracks.iter().enumerate() {
            let clip = &track.clips[0];

            if current_frame >= clip.offset_frames
                && current_frame < clip.offset_frames + clip.duration_frames
            {
                let properties = clip
                    .property_groups
                    .iter()
                    .map(|group| {
                        group
                            .defaults
                            .iter()
                            .map(|default| ClipPropertyValue {
                                value: default.value,
                                is_overridden: false,
                                targeted_by: None,
                            })
                            .collect()
                    })
                    .collect();

                self.clip_indices.push(Some(self.active_clips.len()));
                self.active_clips.push(ActiveClip {
                    name: String::new(),
                    reference: ClipReference::new(clip.id),
                    track_index,
                    clip_index: 0,
                    local_time: current_frame - clip.offset_frames,
                    properties,
                });
            } else {
                self.clip_indices.push(None);
            }
        }
    }
}

impl ActiveClipMap for PlayerClipMap {
    fn active_clips(&self) -> &[ActiveClip] {
        &self.active_clips
    }

    fn active_clips_mut(&mut self) -> &mut [ActiveClip] {
        &mut self.active_clips
    }

    fn get_clip_index(&self, reference: ClipReference) -> Option<usize> {
        self.clip_indices[reference.clip_id() as usize]
    }
}
