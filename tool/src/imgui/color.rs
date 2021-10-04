use imgui_sys::{igColorConvertFloat4ToU32, ImU32, ImVec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImColor(ImU32);

impl From<ImColor> for ImU32 {
    fn from(color: ImColor) -> Self {
        color.0
    }
}

impl From<ImU32> for ImColor {
    fn from(color: ImU32) -> Self {
        ImColor(color)
    }
}

impl From<ImVec4> for ImColor {
    fn from(v: ImVec4) -> Self {
        ImColor(unsafe { igColorConvertFloat4ToU32(v) })
    }
}

impl From<[f32; 4]> for ImColor {
    fn from(v: [f32; 4]) -> Self {
        ImColor(unsafe { igColorConvertFloat4ToU32(v.into()) })
    }
}

impl From<(f32, f32, f32, f32)> for ImColor {
    fn from(v: (f32, f32, f32, f32)) -> Self {
        ImColor(unsafe { igColorConvertFloat4ToU32(v.into()) })
    }
}

impl From<[f32; 3]> for ImColor {
    fn from(v: [f32; 3]) -> Self {
        [v[0], v[1], v[2], 1.].into()
    }
}

impl From<(f32, f32, f32)> for ImColor {
    fn from(v: (f32, f32, f32)) -> Self {
        (v.0, v.1, v.2, 1.).into()
    }
}
