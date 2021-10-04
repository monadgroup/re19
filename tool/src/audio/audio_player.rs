pub trait AudioPlayer {
    fn update(&self);
    fn get_current_seconds(&self) -> f64;
}

pub trait ControllableAudioPlayer: AudioPlayer {
    fn play(&mut self);
    fn pause(&mut self);
    fn seek(&mut self, seconds: f64);
}
