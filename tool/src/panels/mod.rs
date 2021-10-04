mod motion_editor;
mod preview;
mod profiler;
mod property_editor;
//mod recycle_bin;
//mod mesh_editor;
mod time_bar;
mod timeline;

pub use self::motion_editor::draw_motion_editor;
pub use self::preview::draw_preview;
pub use self::profiler::draw_profiler;
pub use self::property_editor::draw_property_editor;
//pub use self::recycle_bin::draw_recycle_bin;
//pub use self::mesh_editor::draw_mesh_editor;
pub use self::time_bar::{draw_time_bar, SCRUBBER_HEIGHT};
pub use self::timeline::draw_timeline;

const ZOOM_CURVE_AMOUNT: f32 = 1000.;
const ZOOM_MIN_FACTOR: f32 = 1000. / 600.;
const ZOOM_MAX_FACTOR: f32 = 1000.;

pub fn get_fpb(fps: f32, bpm: f32) -> f32 {
    (fps * 60.) / bpm
}

pub fn zoom_to_time_scale(zoom: f32, fpb: f32) -> f32 {
    // Zoom goes from 0 to 1
    // At zoom = 0, ten minute = 1000 pixels (so frames * 1000/60 / fps)
    // At zoom = 1, one second = 1000 pixels (so frames * 1000 / fps)
    let curve_progress = (ZOOM_CURVE_AMOUNT.powf(zoom) - 1.) / (ZOOM_CURVE_AMOUNT - 1.);
    let factor = ZOOM_MIN_FACTOR + (ZOOM_MAX_FACTOR - ZOOM_MIN_FACTOR) * curve_progress;
    factor / fpb
}
