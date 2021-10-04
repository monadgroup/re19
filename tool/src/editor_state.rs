use crate::audio::ControllableAudioPlayer;
use engine::animation::clip::ClipReference;
use engine::animation::property::PropertyValue;
use engine::animation::schema::GeneratorSchema;
use imgui_sys::{ImGuiID, ImVec2};

pub struct EditorState<'player> {
    pub fps: f32,
    pub bpm: f32,
    pub beats_per_bar: u32,
    pub timeline_zoom: f32,
    pub motion_editor_zoom: f32,
    pub motion_editor_pan: ImVec2,

    pub drag_offset: i32,
    pub track_pixel_offset: f32,
    pub track_offset: i32,
    pub snapping_points: Vec<i32>,
    pub drag_start_position: i32,

    pub next_clip_id: u32,

    pub insert_clip_schema: Option<&'static GeneratorSchema>,
    pub insert_clip_properties: Option<Vec<Vec<PropertyValue>>>,
    pub just_inserted_clip: Option<u32>,

    pub select_clip_request: Option<ImGuiID>,
    pub select_clip_response: Option<ClipReference>,

    pub renaming_clip: Option<ClipReference>,
    pub retarget_clip_request: Option<(&'static GeneratorSchema, ClipReference)>,
    pub retarget_clip_response: Option<ClipReference>,

    pub insert_animation: Option<(
        ClipReference,
        &'static GeneratorSchema,
        usize,
        usize,
        PropertyValue,
        u32,
    )>,

    pub cam_locked: Option<ImVec2>,

    current_frame: u32,
    is_playing: bool,
    audio_player: &'player mut ControllableAudioPlayer,
}

impl<'player> EditorState<'player> {
    pub fn new(
        fps: f32,
        bpm: f32,
        beats_per_bar: u32,
        audio_player: &'player mut ControllableAudioPlayer,
    ) -> Self {
        EditorState {
            fps,
            bpm,
            beats_per_bar,
            drag_offset: 0,
            track_pixel_offset: 0.,
            track_offset: 0,
            snapping_points: Vec::new(),
            timeline_zoom: 0.,
            drag_start_position: 0,
            next_clip_id: 0,
            insert_clip_schema: None,
            insert_clip_properties: None,
            just_inserted_clip: None,
            motion_editor_zoom: 0.,
            motion_editor_pan: ImVec2::new(0., 0.),
            insert_animation: None,
            select_clip_request: None,
            select_clip_response: None,
            renaming_clip: None,
            current_frame: 0,
            is_playing: false,
            cam_locked: None,
            retarget_clip_request: None,
            retarget_clip_response: None,
            audio_player,
        }
    }

    pub fn frame_to_seconds(&self, frame: u32) -> f32 {
        frame as f32 / self.fps
    }

    pub fn seconds_to_frame(&self, secs: f32) -> u32 {
        (secs * self.fps) as u32
    }

    pub fn update(&mut self) {
        self.current_frame = self.seconds_to_frame(self.audio_player.get_current_seconds() as f32);
    }

    pub fn post_update(&self) {
        self.audio_player.update();
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_frame(&self) -> u32 {
        self.current_frame
    }

    pub fn play(&mut self) {
        if !self.is_playing {
            self.is_playing = true;
            //self.seek_to_frame(self.current_frame);
            self.audio_player.play();
        }
    }

    pub fn pause(&mut self) {
        if self.is_playing {
            self.is_playing = false;
            self.audio_player.pause();
            //self.seek_to_frame(self.current_frame);
        }
    }

    pub fn play_pause(&mut self) {
        if self.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn seek_to_frame(&mut self, frame: u32) {
        self.audio_player.seek(self.frame_to_seconds(frame) as f64);
    }

    pub fn seek_relative(&mut self, relative_frame: i32) {
        let new_frame = (self.current_frame as i32 + relative_frame).max(0);
        self.seek_to_frame(new_frame as u32);
    }
}
