use crate::cstr;
use crate::editor_state::EditorState;
use crate::imgui::DrawList;
use imgui_sys::{
    igGetCursorPosY, igGetCursorScreenPos_nonUDT2, igInvisibleButton, igSetCursorPosY, ImVec2,
};

pub const SCRUBBER_HEIGHT: f32 = 20.;
const MARKER_DIVIDER: f32 = 2.;

pub fn draw_time_bar(
    fpb: f32,
    start_beats: f32,
    end_beats: f32,
    pixel_width: f32,
    extend_lines_height: f32,
    editor_state: &EditorState,
) {
    if pixel_width == 0. {
        return;
    }

    let time_scale = pixel_width / ((end_beats - start_beats) * fpb);

    let screen_top_left = unsafe { igGetCursorScreenPos_nonUDT2() };
    let cursor_start_y = unsafe { igGetCursorPosY() };

    // Make an invisible button that will be our 'control'
    unsafe { igInvisibleButton(cstr!("scrubber"), ImVec2::new(pixel_width, SCRUBBER_HEIGHT)) };

    let mut draw_list = DrawList::for_current_window();
    draw_list.draw_line(
        (screen_top_left.x, screen_top_left.y + SCRUBBER_HEIGHT - 1.),
        (
            screen_top_left.x + pixel_width,
            screen_top_left.y + SCRUBBER_HEIGHT - 1.,
        ),
        (0.3, 0.3, 0.3),
        1.,
    );

    let mark_beats =
        MARKER_DIVIDER.powf(((80. / (time_scale * fpb)).log2() / MARKER_DIVIDER.log2()).ceil());
    let first_marker_index = (start_beats / mark_beats).floor();
    let last_marker_index = (end_beats / mark_beats).ceil();

    for marker_index in (first_marker_index as i32)..(last_marker_index as i32) {
        let marker_beats = marker_index as f32 * mark_beats;
        let marker_pixels = marker_beats * time_scale * fpb;
        let global_marker_x =
            (screen_top_left.x - start_beats * time_scale * fpb + marker_pixels).floor();

        draw_list.draw_line(
            (global_marker_x, screen_top_left.y),
            (global_marker_x, screen_top_left.y + SCRUBBER_HEIGHT - 1.),
            (0.3, 0.3, 0.3),
            1.,
        );

        if extend_lines_height > 0. {
            draw_list.draw_line(
                (global_marker_x, screen_top_left.y + SCRUBBER_HEIGHT),
                (
                    global_marker_x,
                    screen_top_left.y + SCRUBBER_HEIGHT + extend_lines_height,
                ),
                (0.2, 0.2, 0.2),
                1.,
            );
        }

        let current_bar = marker_beats as i32 / editor_state.beats_per_bar as i32;
        let current_beat = marker_beats as i32 % editor_state.beats_per_bar as i32;
        let marker_label = if marker_beats >= 0. {
            if marker_beats.fract() > 0.001 {
                format!(
                    "{:02}:{:02}.{:02}",
                    current_bar,
                    current_beat,
                    (marker_beats.fract() * 100.) as i32
                )
            } else {
                format!("{:02}:{:02}", current_bar, current_beat)
            }
        } else {
            if marker_beats.fract() < -0.001 {
                format!(
                    "-{:02}:{:02}.{:02}",
                    -current_bar,
                    -current_beat,
                    -(marker_beats.fract() * 100.) as i32
                )
            } else {
                format!("-{:02}:{:02}", -current_bar, -current_beat)
            }
        };

        draw_list.draw_text(
            (
                global_marker_x + 5.,
                screen_top_left.y + SCRUBBER_HEIGHT - 18.,
            ),
            (1., 1., 1.),
            &marker_label,
        );
    }

    unsafe { igSetCursorPosY(cursor_start_y + SCRUBBER_HEIGHT) };
}
