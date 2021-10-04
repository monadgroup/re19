use super::{AudioPlayer, ControllableAudioPlayer};
use std::time::Instant;

pub struct NoAudioPlayer {
    start_time: f64,
    playing: Option<Instant>,
}

impl NoAudioPlayer {
    pub fn new() -> NoAudioPlayer {
        NoAudioPlayer {
            start_time: 0.,
            playing: None,
        }
    }
}

impl AudioPlayer for NoAudioPlayer {
    fn update(&self) {}

    fn get_current_seconds(&self) -> f64 {
        match self.playing {
            Some(play_start_time) => self.start_time + play_start_time.elapsed().as_secs_f64(),
            None => self.start_time,
        }
    }
}

impl ControllableAudioPlayer for NoAudioPlayer {
    fn play(&mut self) {
        if self.playing.is_none() {
            self.playing = Some(Instant::now());
        }
    }

    fn pause(&mut self) {
        if self.playing.is_some() {
            self.start_time = self.get_current_seconds();
            self.playing = None;
        }
    }

    fn seek(&mut self, seconds: f64) {
        self.start_time = seconds;
        if self.playing.is_some() {
            self.playing = Some(Instant::now());
        }
    }
}
