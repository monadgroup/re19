use crate::cstr;
use crate::imgui::DrawList;
use engine::resources::perf_table::PerfTable;
use imgui_sys::{
    igBegin, igEnd, igGetContentRegionAvail_nonUDT2, igGetCursorScreenPos_nonUDT2, igPopStyleVar,
    igPushStyleVarVec2, ImGuiStyleVar, ImGuiWindowFlags, ImVec2,
};
use std::ptr;

const BAR_HEIGHT: f32 = 15.;
const BAR_PADDING: f32 = 1.;
const PROFILE_COLORS: &'static [(f32, f32, f32)] = &[
    (0.102, 0.737, 0.612),
    (0.608, 0.349, 0.714),
    (0.204, 0.596, 0.859),
    (0.902, 0.494, 0.133),
    (0.906, 0.298, 0.235),
    (0.953, 0.612, 0.071),
    (0.180, 0.800, 0.443),
];

pub fn draw_profiler(perf: &mut PerfTable) {
    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));
    }
    let show_window = unsafe {
        igBegin(
            cstr!("Profiler"),
            ptr::null_mut(),
            ImGuiWindowFlags::empty(),
        )
    };
    unsafe {
        igPopStyleVar(1);
    }

    if show_window {
        let screen_pos = unsafe { igGetCursorScreenPos_nonUDT2() };
        let available_size = unsafe { igGetContentRegionAvail_nonUDT2() };
        let mut draw_list = DrawList::for_current_window();

        //let target_ms = 1000. / editor_state.fps;
        //let width_ms = target_ms * 2.;
        let width_ms = 1000. / 24.;

        for (perf_index, perf_result) in perf.last_results().into_iter().enumerate() {
            let color_index = perf_index % PROFILE_COLORS.len();

            let start_x = screen_pos.x + perf_result.start_ms / width_ms * available_size.x;
            let end_x = screen_pos.x + perf_result.end_ms / width_ms * available_size.x;
            let rect_y = screen_pos.y + perf_index as f32 * (BAR_HEIGHT + BAR_PADDING);
            if start_x != end_x {
                draw_list
                    .rect((start_x, rect_y), (end_x, rect_y + BAR_HEIGHT))
                    .fill(PROFILE_COLORS[color_index])
                    .draw();
            }

            let text = format!(
                "{} ({:.2}ms)",
                perf_result.name,
                perf_result.end_ms - perf_result.start_ms
            );
            draw_list.draw_text((end_x + 5., rect_y + 1.), (1., 1., 1.), &text);
        }
    }
    unsafe {
        igEnd();
    }
}
