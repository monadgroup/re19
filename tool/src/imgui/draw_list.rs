#![allow(dead_code)]

use super::ImColor;
use imgui_sys::{
    igGetFont, igGetOverlayDrawList, igGetWindowDrawList, ImDrawCornerFlags, ImDrawList,
    ImDrawList_AddCircle, ImDrawList_AddCircleFilled, ImDrawList_AddConvexPolyFilled,
    ImDrawList_AddImage, ImDrawList_AddLine, ImDrawList_AddPolyLine, ImDrawList_AddQuad,
    ImDrawList_AddQuadFilled, ImDrawList_AddRect, ImDrawList_AddRectFilled,
    ImDrawList_AddRectFilledMultiColor, ImDrawList_AddText, ImDrawList_AddTriangle,
    ImDrawList_AddTriangleFilled, ImDrawList_PrimQuadUV, ImDrawList_PrimReserve, ImFont_FindGlyph,
    ImTextureID, ImVec2,
};
use std::os::raw::c_char;

pub struct DrawList {
    draw_list: *mut ImDrawList,
}

impl DrawList {
    pub fn for_current_window() -> DrawList {
        DrawList {
            draw_list: unsafe { igGetWindowDrawList() },
        }
    }

    pub fn for_overlay() -> DrawList {
        DrawList {
            draw_list: unsafe { igGetOverlayDrawList() },
        }
    }

    pub fn draw_line(
        &mut self,
        start: impl Into<ImVec2>,
        end: impl Into<ImVec2>,
        color: impl Into<ImColor>,
        thickness: f32,
    ) {
        unsafe {
            ImDrawList_AddLine(
                self.draw_list,
                start.into(),
                end.into(),
                color.into().into(),
                thickness,
            )
        }
    }

    pub fn rect(
        &mut self,
        top_left: impl Into<ImVec2>,
        bottom_right: impl Into<ImVec2>,
    ) -> DrawRect {
        DrawRect::new(self, top_left.into(), bottom_right.into())
    }

    pub fn quad(
        &mut self,
        a: impl Into<ImVec2>,
        b: impl Into<ImVec2>,
        c: impl Into<ImVec2>,
        d: impl Into<ImVec2>,
    ) -> DrawQuad {
        DrawQuad::new(self, a.into(), b.into(), c.into(), d.into())
    }

    pub fn triangle(
        &mut self,
        a: impl Into<ImVec2>,
        b: impl Into<ImVec2>,
        c: impl Into<ImVec2>,
    ) -> DrawTriangle {
        DrawTriangle::new(self, a.into(), b.into(), c.into())
    }

    pub fn circle(&mut self, centre: impl Into<ImVec2>, radius: f32) -> DrawCircle {
        DrawCircle::new(self, centre.into(), radius)
    }

    pub fn draw_text(&mut self, pos: impl Into<ImVec2>, color: impl Into<ImColor>, text: &str) {
        let text_begin = text.as_ptr() as *const c_char;
        let text_end = unsafe { text_begin.add(text.len()) };
        unsafe {
            ImDrawList_AddText(
                self.draw_list,
                pos.into(),
                color.into().into(),
                text_begin,
                text_end,
            );
        }
    }

    pub fn draw_vertical_text(
        &mut self,
        pos: impl Into<ImVec2>,
        color: impl Into<ImColor>,
        text: &str,
    ) {
        let font = unsafe { igGetFont() };
        let rcolor = color.into().into();
        let mut rpos = pos.into();
        for character in text.chars() {
            let glyph = unsafe { ImFont_FindGlyph(font, character as _) };
            if glyph.is_null() {
                continue;
            }

            unsafe {
                ImDrawList_PrimReserve(self.draw_list, 6, 4);
                ImDrawList_PrimQuadUV(
                    self.draw_list,
                    (rpos.x + (*glyph).y0, rpos.y - (*glyph).x0).into(),
                    (rpos.x + (*glyph).y0, rpos.y - (*glyph).x1).into(),
                    (rpos.x + (*glyph).y1, rpos.y - (*glyph).x1).into(),
                    (rpos.x + (*glyph).y1, rpos.y - (*glyph).x0).into(),
                    ImVec2::new((*glyph).u0, (*glyph).v0),
                    ImVec2::new((*glyph).u1, (*glyph).v0),
                    ImVec2::new((*glyph).u1, (*glyph).v1),
                    ImVec2::new((*glyph).u0, (*glyph).v1),
                    rcolor,
                );
            }

            rpos.y -= unsafe { (*glyph).advance_x };
        }
    }

    pub fn image(
        &mut self,
        texture_id: ImTextureID,
        a: impl Into<ImVec2>,
        b: impl Into<ImVec2>,
        background: impl Into<ImColor>,
    ) -> DrawTexture {
        DrawTexture::new(self, texture_id, a.into(), b.into(), background.into())
    }

    pub fn draw_polyline(
        &mut self,
        points: &[ImVec2],
        color: impl Into<ImColor>,
        closed: bool,
        thickness: f32,
    ) {
        unsafe {
            ImDrawList_AddPolyLine(
                self.draw_list,
                &points[0],
                points.len() as i32,
                color.into().into(),
                closed,
                thickness,
            );
        }
    }

    pub fn polygon<'list, 'points>(
        &'list mut self,
        points: &'points [ImVec2],
    ) -> DrawPolygon<'list, 'points> {
        DrawPolygon::new(self, points)
    }
}

pub struct DrawRect<'list> {
    draw_list: &'list mut DrawList,
    top_left: ImVec2,
    bottom_right: ImVec2,
    rounding: f32,
    rounding_flags: ImDrawCornerFlags,
    fill: FillMode,
    stroke: Option<(ImColor, f32)>,
}

enum FillMode {
    None,
    SingleColor(ImColor),
    MultiColor {
        top_left: ImColor,
        top_right: ImColor,
        bottom_right: ImColor,
        bottom_left: ImColor,
    },
}

impl<'list> DrawRect<'list> {
    fn new(draw_list: &'list mut DrawList, top_left: ImVec2, bottom_right: ImVec2) -> Self {
        DrawRect {
            draw_list,
            top_left,
            bottom_right,
            rounding: 0.,
            rounding_flags: ImDrawCornerFlags::empty(),
            fill: FillMode::None,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: impl Into<ImColor>) -> Self {
        self.fill = FillMode::SingleColor(color.into());
        self
    }

    pub fn multi_fill(
        mut self,
        top_left: impl Into<ImColor>,
        top_right: impl Into<ImColor>,
        bottom_right: impl Into<ImColor>,
        bottom_left: impl Into<ImColor>,
    ) -> Self {
        self.fill = FillMode::MultiColor {
            top_left: top_left.into(),
            top_right: top_right.into(),
            bottom_right: bottom_right.into(),
            bottom_left: bottom_left.into(),
        };
        self
    }

    pub fn stroke(mut self, color: impl Into<ImColor>, thickness: f32) -> Self {
        self.stroke = Some((color.into(), thickness));
        self
    }

    pub fn round(
        mut self,
        rounding: f32,
        top_left: bool,
        top_right: bool,
        bottom_right: bool,
        bottom_left: bool,
    ) -> Self {
        self.rounding = rounding;
        self.rounding_flags = ImDrawCornerFlags::empty();

        if top_left {
            self.rounding_flags |= ImDrawCornerFlags::TopLeft;
        }
        if top_right {
            self.rounding_flags |= ImDrawCornerFlags::TopRight;
        }
        if bottom_right {
            self.rounding_flags |= ImDrawCornerFlags::BotRight;
        }
        if bottom_left {
            self.rounding_flags |= ImDrawCornerFlags::BotLeft;
        }

        self
    }

    pub fn round_all(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self.rounding_flags = ImDrawCornerFlags::All;
        self
    }

    pub fn round_top(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self.rounding_flags = ImDrawCornerFlags::Top;
        self
    }

    pub fn round_bottom(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self.rounding_flags = ImDrawCornerFlags::Bot;
        self
    }

    pub fn round_left(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self.rounding_flags = ImDrawCornerFlags::Left;
        self
    }

    pub fn round_right(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self.rounding_flags = ImDrawCornerFlags::Right;
        self
    }

    pub fn draw(self) {
        match self.fill {
            FillMode::SingleColor(col) => unsafe {
                ImDrawList_AddRectFilled(
                    self.draw_list.draw_list,
                    self.top_left,
                    self.bottom_right,
                    col.into(),
                    self.rounding,
                    self.rounding_flags,
                )
            },
            FillMode::MultiColor {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            } => unsafe {
                ImDrawList_AddRectFilledMultiColor(
                    self.draw_list.draw_list,
                    self.top_left,
                    self.bottom_right,
                    top_left.into(),
                    top_right.into(),
                    bottom_right.into(),
                    bottom_left.into(),
                )
            },
            FillMode::None => {}
        }

        if let Some((stroke_col, stroke_thickness)) = self.stroke {
            unsafe {
                ImDrawList_AddRect(
                    self.draw_list.draw_list,
                    self.top_left,
                    self.bottom_right,
                    stroke_col.into(),
                    self.rounding,
                    self.rounding_flags,
                    stroke_thickness,
                )
            }
        }
    }
}

pub struct DrawQuad<'list> {
    draw_list: &'list mut DrawList,
    a: ImVec2,
    b: ImVec2,
    c: ImVec2,
    d: ImVec2,
    fill: Option<ImColor>,
    stroke: Option<(ImColor, f32)>,
}

impl<'list> DrawQuad<'list> {
    fn new(draw_list: &'list mut DrawList, a: ImVec2, b: ImVec2, c: ImVec2, d: ImVec2) -> Self {
        DrawQuad {
            draw_list,
            a,
            b,
            c,
            d,
            fill: None,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: impl Into<ImColor>) -> Self {
        self.fill = Some(color.into());
        self
    }

    pub fn stroke(mut self, color: impl Into<ImColor>, thickness: f32) -> Self {
        self.stroke = Some((color.into(), thickness));
        self
    }

    pub fn draw(self) {
        if let Some(col) = self.fill {
            unsafe {
                ImDrawList_AddQuadFilled(
                    self.draw_list.draw_list,
                    self.a,
                    self.b,
                    self.c,
                    self.d,
                    col.into(),
                );
            }
        }

        if let Some((col, thickness)) = self.stroke {
            unsafe {
                ImDrawList_AddQuad(
                    self.draw_list.draw_list,
                    self.a,
                    self.b,
                    self.c,
                    self.d,
                    col.into(),
                    thickness,
                );
            }
        }
    }
}

pub struct DrawTriangle<'list> {
    draw_list: &'list mut DrawList,
    a: ImVec2,
    b: ImVec2,
    c: ImVec2,
    fill: Option<ImColor>,
    stroke: Option<(ImColor, f32)>,
}

impl<'list> DrawTriangle<'list> {
    fn new(draw_list: &'list mut DrawList, a: ImVec2, b: ImVec2, c: ImVec2) -> Self {
        DrawTriangle {
            draw_list,
            a,
            b,
            c,
            fill: None,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: impl Into<ImColor>) -> Self {
        self.fill = Some(color.into());
        self
    }

    pub fn stroke(mut self, color: impl Into<ImColor>, thickness: f32) -> Self {
        self.stroke = Some((color.into(), thickness));
        self
    }

    pub fn draw(self) {
        if let Some(col) = self.fill {
            unsafe {
                ImDrawList_AddTriangleFilled(
                    self.draw_list.draw_list,
                    self.a,
                    self.b,
                    self.c,
                    col.into(),
                );
            }
        }

        if let Some((col, thickness)) = self.stroke {
            unsafe {
                ImDrawList_AddTriangle(
                    self.draw_list.draw_list,
                    self.a,
                    self.b,
                    self.c,
                    col.into(),
                    thickness,
                );
            }
        }
    }
}

pub struct DrawCircle<'list> {
    draw_list: &'list mut DrawList,
    centre: ImVec2,
    radius: f32,
    num_segments: u32,
    fill: Option<ImColor>,
    stroke: Option<(ImColor, f32)>,
}

impl<'list> DrawCircle<'list> {
    fn new(draw_list: &'list mut DrawList, centre: ImVec2, radius: f32) -> Self {
        DrawCircle {
            draw_list,
            centre,
            radius,
            num_segments: 12,
            fill: None,
            stroke: None,
        }
    }

    pub fn num_segments(mut self, segments: u32) -> Self {
        self.num_segments = segments;
        self
    }

    pub fn fill(mut self, color: impl Into<ImColor>) -> Self {
        self.fill = Some(color.into());
        self
    }

    pub fn stroke(mut self, color: impl Into<ImColor>, thickness: f32) -> Self {
        self.stroke = Some((color.into(), thickness));
        self
    }

    pub fn draw(self) {
        if let Some(col) = self.fill {
            unsafe {
                ImDrawList_AddCircleFilled(
                    self.draw_list.draw_list,
                    self.centre,
                    self.radius,
                    col.into(),
                    self.num_segments as i32,
                );
            }
        }
        if let Some((col, thickness)) = self.stroke {
            unsafe {
                ImDrawList_AddCircle(
                    self.draw_list.draw_list,
                    self.centre,
                    self.radius,
                    col.into(),
                    self.num_segments as i32,
                    thickness,
                );
            }
        }
    }
}

pub struct DrawTexture<'list> {
    draw_list: &'list mut DrawList,
    texture_id: ImTextureID,
    a: ImVec2,
    b: ImVec2,
    uv_a: ImVec2,
    uv_b: ImVec2,
    background: ImColor,
}

impl<'list> DrawTexture<'list> {
    fn new(
        draw_list: &'list mut DrawList,
        texture_id: ImTextureID,
        a: ImVec2,
        b: ImVec2,
        background: ImColor,
    ) -> DrawTexture {
        DrawTexture {
            draw_list,
            texture_id,
            a,
            b,
            uv_a: ImVec2::new(0., 0.),
            uv_b: ImVec2::new(1., 1.),
            background,
        }
    }

    pub fn uv(mut self, uv_a: impl Into<ImVec2>, uv_b: impl Into<ImVec2>) -> Self {
        self.uv_a = uv_a.into();
        self.uv_b = uv_b.into();
        self
    }

    pub fn draw(self) {
        unsafe {
            ImDrawList_AddImage(
                self.draw_list.draw_list,
                self.texture_id,
                self.a,
                self.b,
                self.uv_a,
                self.uv_b,
                self.background.into(),
            );
        }
    }
}

pub struct DrawPolygon<'list, 'points> {
    draw_list: &'list mut DrawList,
    points: &'points [ImVec2],
    fill: Option<ImColor>,
    stroke: Option<(ImColor, f32)>,
}

impl<'list, 'points> DrawPolygon<'list, 'points> {
    fn new(draw_list: &'list mut DrawList, points: &'points [ImVec2]) -> Self {
        DrawPolygon {
            draw_list,
            points,
            fill: None,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: impl Into<ImColor>) -> Self {
        self.fill = Some(color.into());
        self
    }

    pub fn stroke(mut self, color: impl Into<ImColor>, thickness: f32) -> Self {
        self.stroke = Some((color.into(), thickness));
        self
    }

    pub fn draw(self) {
        if let Some(col) = self.fill {
            unsafe {
                ImDrawList_AddConvexPolyFilled(
                    self.draw_list.draw_list,
                    &self.points[0],
                    self.points.len() as i32,
                    col.into(),
                );
            }
        }
        if let Some((col, thickness)) = self.stroke {
            unsafe {
                ImDrawList_AddPolyLine(
                    self.draw_list.draw_list,
                    &self.points[0],
                    self.points.len() as i32,
                    col.into(),
                    true,
                    thickness,
                );
            }
        }
    }
}
