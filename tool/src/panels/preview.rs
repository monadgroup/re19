use crate::cstr;
use crate::editor_state::EditorState;
use crate::imgui::DrawList;
use engine::animation::clip::{ActiveClipMap, ClipPropertyValue, ClipReference, GeneratorClipMap};
use engine::animation::property::PropertyValue;
use engine::animation::timeline::{ClipSource, Timeline};
use engine::frame_context::{CommonData, FrameContext, FrameDataBuffer};
use engine::gbuffer::GBuffer;
use engine::generator::Generator;
use engine::math::{Quaternion, Vector2, Vector3};
use engine::renderer::RendererCollection;
use engine::resources::perf_table::PerfTable;
use engine::resources::shader_manager::ShaderManager;
use engine::texture::ShaderResource2D;
use engine::viewport::Viewport;
use imgui_sys::{
    igBegin, igEnd, igGetContentRegionAvail_nonUDT2, igGetCursorScreenPos_nonUDT2, igGetIO,
    igIsKeyDown, igIsMouseClicked, igIsWindowHovered, igPopStyleColor, igPopStyleVar,
    igPushStyleColor, igPushStyleVarVec2, igSetMouseCursor, ImGuiCol, ImGuiHoveredFlags,
    ImGuiMouseCursor, ImGuiStyleVar, ImGuiWindowFlags, ImVec2, ImVec4,
};
use std::collections::HashMap;
use std::{mem, ptr};
use winapi::um::d3d11::ID3D11DeviceContext;
use winapi::um::winuser::{GetCursorPos, SetCursorPos, VK_ESCAPE, VK_SHIFT};

const SLOW_SPEED: f32 = 5. / 60.;
const FAST_SPEED: f32 = 50. / 60.;

struct EditorGeneratorClipMap<'gen> {
    generators: HashMap<u32, &'gen mut dyn Generator>,
}

impl<'gen> EditorGeneratorClipMap<'gen> {
    pub fn new(generators: HashMap<u32, &'gen mut dyn Generator>) -> Self {
        EditorGeneratorClipMap { generators }
    }

    pub fn take<F: FnOnce(&mut dyn Generator, &mut EditorGeneratorClipMap<'gen>)>(
        &mut self,
        id: ClipReference,
        func: F,
    ) {
        if let Some(taken_entry) = self.generators.remove(&id.clip_id()) {
            func(taken_entry, self);
            self.generators.insert(id.clip_id(), taken_entry);
        }
    }
}

impl<'gen> GeneratorClipMap for EditorGeneratorClipMap<'gen> {
    fn try_get_clip(&self, reference: ClipReference) -> Option<&dyn Generator> {
        self.generators
            .get(&reference.clip_id())
            .map(|gen| *gen as &dyn Generator)
    }

    fn try_get_clip_mut(&mut self, reference: ClipReference) -> Option<&mut dyn Generator> {
        self.generators
            .get_mut(&reference.clip_id())
            .map(|gen| *gen as &mut dyn Generator)
    }
}

fn get_global_cursor_pos() -> ImVec2 {
    unsafe {
        let mut cursor_pos = mem::zeroed();
        GetCursorPos(&mut cursor_pos);
        ImVec2::new(cursor_pos.x as f32, cursor_pos.y as f32)
    }
}

pub fn draw_preview(
    devcon: *mut ID3D11DeviceContext,
    delta_seconds: f32,
    viewport: Viewport,
    shader_manager: &ShaderManager,
    timeline: &mut Timeline,
    active_map: &ActiveClipMap,
    renderers: &mut RendererCollection,
    buffer: &mut GBuffer,
    common: &mut CommonData,
    perf: &mut PerfTable,
    editor_state: &mut EditorState,
) {
    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));
        igPushStyleColor(ImGuiCol::WindowBg, ImVec4::new(0., 0., 0., 1.));
    };
    let show_window =
        unsafe { igBegin(cstr!("Preview"), ptr::null_mut(), ImGuiWindowFlags::empty()) };
    unsafe {
        igPopStyleColor(1);
        igPopStyleVar(1);
    }

    if show_window {
        // If the user is clicking the window, we can lock it
        if unsafe { igIsWindowHovered(ImGuiHoveredFlags::empty()) && igIsMouseClicked(0, false) } {
            if editor_state.cam_locked.is_some() {
                editor_state.cam_locked = None;
            } else {
                editor_state.cam_locked = Some(get_global_cursor_pos());
            }
        }

        if let Some(start_cursor_pos) = editor_state.cam_locked {
            if unsafe { igIsKeyDown(VK_ESCAPE) } {
                editor_state.cam_locked = None;
            }

            let new_cursor_pos = get_global_cursor_pos();

            // Reset the cursor position to the initial one
            unsafe {
                igSetMouseCursor(ImGuiMouseCursor::None);
                SetCursorPos(start_cursor_pos.x as i32, start_cursor_pos.y as i32);
            }

            let cursor_delta_x = new_cursor_pos.x - start_cursor_pos.x;
            let cursor_delta_y = new_cursor_pos.y - start_cursor_pos.y;
            let yaw_rotation = Quaternion::euler(cursor_delta_x / 150., 0., 0.);
            let pitch_rotation = Quaternion::euler(0., cursor_delta_y / 150., 0.);

            let mut position_delta = Vector3::default();
            let mut roll_delta = 0.;
            if unsafe { igIsKeyDown('A' as i32) } {
                position_delta.x -= 1.;
            }
            if unsafe { igIsKeyDown('D' as i32) } {
                position_delta.x += 1.;
            }
            if unsafe { igIsKeyDown('S' as i32) } {
                position_delta.z -= 1.;
            }
            if unsafe { igIsKeyDown('W' as i32) } {
                position_delta.z += 1.;
            }
            if unsafe { igIsKeyDown('Q' as i32) } {
                position_delta.y -= 1.;
            }
            if unsafe { igIsKeyDown('E' as i32) } {
                position_delta.y += 1.;
            }
            if unsafe { igIsKeyDown('Z' as i32) } {
                roll_delta -= 1.;
            }
            if unsafe { igIsKeyDown('C' as i32) } {
                roll_delta += 1.;
            }
            let roll_rotation = Quaternion::euler(0., 0., roll_delta / 10.);

            let position_delta = position_delta
                * if unsafe { igIsKeyDown(VK_SHIFT) } {
                    FAST_SPEED
                } else {
                    SLOW_SPEED
                };

            let wheel_delta = unsafe { (*igGetIO()).mouse_wheel };
            let fov_delta = wheel_delta * 2.;

            // Update any clips that expose a camera binding
            for active_clip in active_map.active_clips() {
                let clip =
                    &mut timeline.tracks[active_clip.track_index].clips[active_clip.clip_index];
                let generator: &mut dyn Generator = match &mut clip.source {
                    ClipSource::Generator(generator) => generator.as_mut(),
                    _ => continue,
                };
                let camera_binding = match generator.camera_binding() {
                    Some(binding) => binding,
                    None => continue,
                };

                let direction_binding = camera_binding.camera_direction_binding();
                let direction_default = &mut clip.property_groups[direction_binding.group].defaults
                    [direction_binding.prop];
                let direction_active =
                    &active_clip.properties[direction_binding.group][direction_binding.prop];

                let new_direction = direction_active.value.into_rotation().unwrap();
                let new_direction = yaw_rotation * new_direction;
                let new_direction = new_direction * pitch_rotation;
                let new_direction = roll_rotation * new_direction;

                direction_default.value = PropertyValue::Rotation(new_direction);
                if direction_active.targeted_by.is_some() {
                    direction_default.is_override = true;
                }

                let position_binding = camera_binding.camera_position_binding();
                let position_default = &mut clip.property_groups[position_binding.group].defaults
                    [position_binding.prop];
                let position_active =
                    &active_clip.properties[position_binding.group][position_binding.prop];

                let new_position =
                    position_active.value.into_vec3().unwrap() + position_delta * new_direction;
                position_default.value = PropertyValue::Vec3(new_position);
                if position_active.targeted_by.is_some() {
                    position_default.is_override = true;
                }

                let fov_binding = camera_binding.camera_fov_binding();
                let fov_default =
                    &mut clip.property_groups[fov_binding.group].defaults[fov_binding.prop];
                let fov_active = &active_clip.properties[fov_binding.group][fov_binding.prop];

                let new_fov = (fov_active.value.into_float().unwrap() - fov_delta)
                    .max(1.)
                    .min(90.);
                fov_default.value = PropertyValue::Float(new_fov);
                if fov_active.targeted_by.is_some() {
                    fov_default.is_override = true;
                }
            }
        }

        // Update the frame data buffer
        common.frame_data = FrameDataBuffer {
            viewport: Vector2 {
                x: viewport.width as f32,
                y: viewport.height as f32,
            },
            seed: editor_state.frame_to_seconds(editor_state.current_frame()),
        };
        common.frame_data_buffer.upload(devcon, common.frame_data);

        let mut clip_map_map = HashMap::new();
        for track in timeline.tracks.iter_mut() {
            for clip in track.clips.iter_mut() {
                match &mut clip.source {
                    ClipSource::Generator(generator) => {
                        clip_map_map.insert(clip.id, generator.as_mut());
                    }
                    _ => {}
                }
            }
        }
        let mut clip_map = EditorGeneratorClipMap::new(clip_map_map);

        for active_clip in active_map.active_clips() {
            clip_map.take(active_clip.reference, |generator, map| {
                let mut frame_context = FrameContext {
                    devcon,
                    delta_seconds,
                    viewport,
                    shader_manager,
                    clip_map: map,
                    common,
                    perf,
                };

                let prop_references: Vec<_> = active_clip
                    .properties
                    .iter()
                    .map(|props| props as &[ClipPropertyValue])
                    .collect();
                let perf = frame_context
                    .perf
                    .start_gpu_string(format!("\"{}\"", active_clip.name));
                generator.update(
                    buffer,
                    &mut frame_context,
                    renderers,
                    active_clip.local_time,
                    &prop_references,
                );
                frame_context.perf.end(perf);
            });
        }

        let cursor_pos = unsafe { igGetCursorScreenPos_nonUDT2() };
        let preview_area_size = unsafe { igGetContentRegionAvail_nonUDT2() };
        let preview_scale = preview_area_size.x / viewport.width as f32;
        let preview_scale = if preview_scale * viewport.height as f32 > preview_area_size.y {
            preview_area_size.y / viewport.height as f32
        } else {
            preview_scale
        };
        let preview_size = ImVec2::new(
            preview_scale * viewport.width as f32,
            preview_scale * viewport.height as f32,
        );
        let preview_pos = ImVec2::new(
            cursor_pos.x + preview_area_size.x / 2. - preview_size.x / 2.,
            cursor_pos.y + preview_area_size.y / 2. - preview_size.y / 2.,
        );

        let mut draw_list = DrawList::for_current_window();
        draw_list
            .image(
                buffer.write_output().shader_resource_ptr() as *mut _,
                preview_pos,
                (
                    preview_pos.x + preview_size.x,
                    preview_pos.y + preview_size.y,
                ),
                (1., 1., 1., 1.),
            )
            .draw();
        if editor_state.cam_locked.is_some() {
            draw_list.draw_text(
                (cursor_pos.x + 5., cursor_pos.y + 5.),
                (1., 1., 1.),
                "Camera locked",
            );
        }
    }

    unsafe {
        igEnd();
    }
}
