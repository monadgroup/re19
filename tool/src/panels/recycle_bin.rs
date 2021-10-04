use crate::cstr;
use crate::editor_state::EditorState;
use crate::recycle_bin::RecycleBin;
use imgui_sys::{
    igBegin, igEnd, igPopStyleVar, igPushStyleVarVec2, ImGuiStyleVar, ImGuiWindowFlags, ImVec2,
};
use std::ptr;

pub fn draw_recycle_bin(bin: &mut RecycleBin, editor_state: &mut EditorState) {
    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));
    }
    let show_window = unsafe {
        igBegin(
            cstr!("Recycle Bin"),
            ptr::null_mut(),
            ImGuiWindowFlags::empty(),
        )
    };
    unsafe {
        igPopStyleVar(1);
    }

    if show_window {
        // todo
    }

    unsafe {
        igEnd();
    }
}
