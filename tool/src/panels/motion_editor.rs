use super::{draw_time_bar, get_fpb, SCRUBBER_HEIGHT};
use crate::cstr;
use crate::editor_state::EditorState;
use crate::imgui::DrawList;
use crate::timeline_interactions::{delete_keyframe, remove_clip};
use engine::animation::animation_clip::{
    AnimatedPropertyField, AnimatedPropertyTarget, AnimationClip, CurveInterpolation, CurveSegment,
};
use engine::animation::clip::ActiveClipMap;
use engine::animation::cubic_bezier::CubicBezier;
use engine::animation::property::PropertyValue;
use engine::animation::schema::GeneratorSchema;
use engine::animation::timeline::{ClipSource, Timeline};
use engine::math::Vector2;
use imgui_sys::{
    igArrowButton, igBegin, igBeginChild, igBeginPopupContextItem, igButton,
    igCalcTextSize_nonUDT2, igEnd, igEndChild, igEndPopup, igGetContentRegionAvail_nonUDT2,
    igGetCursorPosX, igGetCursorPosY, igGetCursorPos_nonUDT2, igGetCursorScreenPos_nonUDT2,
    igGetIO, igGetMouseDragDelta_nonUDT2, igGetMousePos_nonUDT2, igGetWindowPos_nonUDT2,
    igInvisibleButton, igIsItemActive, igIsItemClicked, igIsItemHovered, igIsMouseDragging,
    igIsRectVisibleVec2, igIsWindowHovered, igMenuItemBool, igPopClipRect, igPopID, igPopStyleVar,
    igPushClipRect, igPushIDInt, igPushStyleVarVec2, igResetMouseDragDelta, igSameLine,
    igSeparator, igSetCursorPos, igSetCursorPosX, igSetCursorPosY, igSetCursorScreenPos,
    igSetNextWindowPos, igSetTooltip, ImGuiCond, ImGuiDir, ImGuiHoveredFlags, ImGuiStyleVar,
    ImGuiWindowFlags, ImVec2, ImVec4,
};
use std::{f32, iter, ptr, slice, u32};

const KEYFRAME_BAR_VIRTUAL_HEIGHT: f32 = 40.;
const MOTION_BAR_VIRTUAL_HEIGHT: f32 = 150.;
const CLIP_NAME_WIDTH: f32 = 20.;
const SIDEBAR_WIDTH: f32 = 150.;

const ZOOM_CURVE_AMOUNT: f32 = 2.;
const ZOOM_MIN_FACTOR: f32 = 0.8;
const ZOOM_MAX_FACTOR: f32 = 10.;
const KEYFRAME_CONTROLLER_WIDTH: f32 = 10.;
const KEYFRAME_CONTROLLER_HEIGHT: f32 = 20.;

const CURVE_PIXEL_SPACE: f32 = 5.;

fn calc_text_width(text: &str) -> f32 {
    let text_start = text.as_ptr() as *const i8;
    let text_end = unsafe { text_start.add(text.len()) };
    unsafe { igCalcTextSize_nonUDT2(text_start, text_end, false, 0.).x }
}

pub fn draw_motion_editor(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    clip_map: &ActiveClipMap,
) {
    let fpb = get_fpb(editor_state.fps, editor_state.bpm);

    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));
    }
    let show_window = unsafe {
        igBegin(
            cstr!("Motion Editor"),
            ptr::null_mut(),
            ImGuiWindowFlags::NoScrollbar | ImGuiWindowFlags::NoScrollWithMouse,
        )
    };
    unsafe {
        igPopStyleVar(1);
    };

    if show_window {
        // Build a list of animation clips to display
        // If there aren't any selected, we just show all of them
        let are_selected_animations = timeline
            .tracks
            .iter()
            .flat_map(|track| track.clips.iter())
            .any(|clip| match &clip.source {
                ClipSource::Animation(_) => clip.is_selected,
                _ => false,
            });

        let selected_clips: Vec<_> = timeline
            .tracks
            .iter_mut()
            .flat_map(|track| {
                track.clips.iter_mut().scan(0, |last_end_time, clip| {
                    let clip_start_time = *last_end_time + clip.offset_frames;
                    *last_end_time = clip_start_time + clip.duration_frames;

                    Some((clip_start_time, clip))
                })
            })
            .filter(|(_, clip)| !are_selected_animations || clip.is_selected)
            .filter_map(|(clip_start_time, clip)| match &mut clip.source {
                ClipSource::Animation(animation) => Some((
                    clip_start_time,
                    clip.duration_frames,
                    &clip.name as &str,
                    clip.schema,
                    animation,
                )),
                _ => None,
            })
            .collect();
        edit_clips(selected_clips, editor_state, clip_map, fpb);

        // Delete any animation clips that are now empty
        // todo: this currently breaks if the clip is active
        let mut delete_clip = None;
        for (track_index, track) in timeline.tracks.iter().enumerate() {
            for (clip_index, clip) in track.clips.iter().enumerate() {
                if let ClipSource::Animation(animation) = &clip.source {
                    if animation.properties.is_empty() {
                        delete_clip = Some((track_index, clip_index));
                        break;
                    }
                }
            }

            if delete_clip.is_some() {
                break;
            }
        }

        if let Some((delete_track_index, delete_clip_index)) = delete_clip {
            remove_clip(&mut timeline.tracks[delete_track_index], delete_clip_index);
            //trim_empty_tracks(timeline);
        }
    }

    unsafe { igEnd() };
}

fn zoom_to_pixel_scale(zoom: f32) -> f32 {
    let curve_progress = (ZOOM_CURVE_AMOUNT.powf(zoom) - 1.) / (ZOOM_CURVE_AMOUNT - 1.);
    ZOOM_MIN_FACTOR + (ZOOM_MAX_FACTOR - ZOOM_MIN_FACTOR) * curve_progress
}

fn edit_clips(
    clips: Vec<(u32, u32, &str, &GeneratorSchema, &mut AnimationClip)>,
    editor_state: &mut EditorState,
    clip_map: &ActiveClipMap,
    fpb: f32,
) {
    let screen_cursor = unsafe { igGetCursorScreenPos_nonUDT2() };
    let available_size = unsafe { igGetContentRegionAvail_nonUDT2() };

    // If the user is zooming, update the scroll
    let last_zoom = if unsafe { igIsWindowHovered(ImGuiHoveredFlags::ChildWindows) } {
        let scroll_delta = unsafe { (*igGetIO()).mouse_wheel };

        if scroll_delta != 0. {
            let last_zoom = editor_state.motion_editor_zoom;
            editor_state.motion_editor_zoom = (editor_state.motion_editor_zoom
                + scroll_delta / 50.)
                .max(0.)
                .min(1.);
            Some(last_zoom)
        } else {
            None
        }
    } else {
        None
    };

    // Draw the sidebar area
    let mut draw_list = DrawList::for_current_window();
    draw_list
        .rect(
            (screen_cursor.x, screen_cursor.y),
            (
                screen_cursor.x + SIDEBAR_WIDTH,
                screen_cursor.y + available_size.y,
            ),
        )
        .fill((0.2, 0.2, 0.2))
        .draw();
    draw_list.draw_line(
        (screen_cursor.x + CLIP_NAME_WIDTH, screen_cursor.y),
        (
            screen_cursor.x + CLIP_NAME_WIDTH,
            screen_cursor.y + available_size.y,
        ),
        (0., 0., 0.),
        1.,
    );

    // If there's nothing to show, early exit
    if clips.is_empty() {
        draw_list.draw_line(
            (screen_cursor.x + SIDEBAR_WIDTH, screen_cursor.y),
            (
                screen_cursor.x + SIDEBAR_WIDTH,
                screen_cursor.y + available_size.y,
            ),
            (0.3, 0.3, 0.3),
            1.,
        );

        return;
    }

    // Calculate the time range of visible clips
    let (min_frames, max_frames) = clips.iter().fold(
        (u32::MAX, u32::MIN),
        |(min_frames, max_frames), (start_frames, duration_frames, _, _, _)| {
            (
                min_frames.min(*start_frames),
                max_frames.max(*start_frames + *duration_frames),
            )
        },
    );
    let virtual_canvas_width = available_size.x - SIDEBAR_WIDTH;
    let time_scale = virtual_canvas_width / (max_frames - min_frames) as f32;

    // Calculate the height of the virtual canvas (the height of all of the properties)
    let virtual_canvas_height = clips
        .iter()
        .map(|(_, _, _, _, clip)| get_clip_virtual_height(clip))
        .sum::<f32>();

    // Determine our current rectangle in the virtual canvases coordinate space
    let pixel_scale = zoom_to_pixel_scale(editor_state.motion_editor_zoom);
    let virtual_viewport_size = ImVec2::new(
        virtual_canvas_width / pixel_scale,
        ((available_size.y - SCRUBBER_HEIGHT) / pixel_scale).min(virtual_canvas_height),
    );
    let max_y_pan =
        (virtual_canvas_height - virtual_viewport_size.y).max(0.) / virtual_canvas_height;

    // If the zoom changed, adjust the pan so the mouse is at the same place
    if let Some(last_zoom) = last_zoom {
        let last_pixel_scale = zoom_to_pixel_scale(last_zoom);
        let mouse_pos = unsafe { igGetMousePos_nonUDT2() };
        let local_mouse_pos = ImVec2::new(
            mouse_pos.x - screen_cursor.x - SIDEBAR_WIDTH,
            mouse_pos.y - screen_cursor.y - SCRUBBER_HEIGHT,
        );

        let last_viewport_pos = ImVec2::new(
            virtual_canvas_width * editor_state.motion_editor_pan.x,
            virtual_canvas_height * editor_state.motion_editor_pan.y,
        );
        let virtual_viewport_pos = ImVec2::new(
            virtual_canvas_width * editor_state.motion_editor_pan.x,
            virtual_canvas_height * editor_state.motion_editor_pan.y,
        );

        // Convert the mouse position to pan-space (i.e {0->1} on both coordinates) with the current
        // scale and the previous scale.
        let last_normal_mouse_pos = ImVec2::new(
            (local_mouse_pos.x / last_pixel_scale + last_viewport_pos.x) / virtual_canvas_width,
            (local_mouse_pos.y / last_pixel_scale + last_viewport_pos.y) / virtual_canvas_height,
        );
        let new_normal_mouse_pos = ImVec2::new(
            (local_mouse_pos.x / pixel_scale + virtual_viewport_pos.x) / virtual_canvas_width,
            (local_mouse_pos.y / pixel_scale + virtual_viewport_pos.y) / virtual_canvas_height,
        );
        editor_state.motion_editor_pan = ImVec2::new(
            editor_state.motion_editor_pan.x + last_normal_mouse_pos.x - new_normal_mouse_pos.x,
            (editor_state.motion_editor_pan.y + last_normal_mouse_pos.y - new_normal_mouse_pos.y)
                .max(0.)
                .min(max_y_pan),
        );
    }

    // If the mouse is being dragged, update the pan
    if unsafe { igIsWindowHovered(ImGuiHoveredFlags::ChildWindows) && igIsMouseDragging(2, 1.) } {
        let drag_delta = unsafe { igGetMouseDragDelta_nonUDT2(2, 1.) };

        let rel_x_delta = drag_delta.x / pixel_scale / virtual_canvas_width;
        let rel_y_delta = drag_delta.y / pixel_scale / virtual_canvas_height;
        editor_state.motion_editor_pan = ImVec2::new(
            editor_state.motion_editor_pan.x - rel_x_delta,
            (editor_state.motion_editor_pan.y - rel_y_delta)
                .max(0.)
                .min(max_y_pan),
        );

        unsafe {
            igResetMouseDragDelta(2);
        }
    }

    // Pan goes from 0-1 in both axis, where 0 means all the way on the left or top, and 1 is on the right or bottom
    let virtual_viewport_pos = ImVec2::new(
        virtual_canvas_width * editor_state.motion_editor_pan.x,
        virtual_canvas_height * editor_state.motion_editor_pan.y,
    );

    let virtual_viewport = ImVec4::new(
        virtual_viewport_pos.x,
        virtual_viewport_pos.y,
        virtual_viewport_pos.x + virtual_viewport_size.x,
        virtual_viewport_pos.y + virtual_viewport_size.y,
    );

    // Display the time bar at the top
    let start_frames = min_frames as f32
        + virtual_viewport.x / virtual_canvas_width * (max_frames - min_frames) as f32;
    let end_frames = min_frames as f32
        + virtual_viewport.z / virtual_canvas_width * (max_frames - min_frames) as f32;
    let start_cursor_x = unsafe { igGetCursorPosX() };
    unsafe {
        igSetCursorPosX(start_cursor_x + SIDEBAR_WIDTH);
        igPushClipRect(
            ImVec2::new(screen_cursor.x + SIDEBAR_WIDTH, screen_cursor.y),
            ImVec2::new(
                screen_cursor.x + available_size.x,
                screen_cursor.y + available_size.y,
            ),
            false,
        );
    };
    draw_time_bar(
        fpb,
        start_frames / fpb,
        end_frames / fpb,
        available_size.x - SIDEBAR_WIDTH,
        available_size.y - SCRUBBER_HEIGHT,
        editor_state,
    );
    if unsafe { igIsItemActive() } {
        let mouse_screen_pos = unsafe { igGetMousePos_nonUDT2() };
        editor_state.seek_to_frame(
            (((mouse_screen_pos.x - SIDEBAR_WIDTH - screen_cursor.x) / pixel_scale
                + virtual_viewport.x)
                / time_scale
                + min_frames as f32)
                .max(0.) as u32,
        );
    }

    unsafe {
        igPopClipRect();
        igSetCursorPosX(start_cursor_x)
    };

    // Display the scrubber if it's visible
    let current_frame = editor_state.current_frame();
    let scrubber_frames = current_frame as f32;
    if scrubber_frames >= start_frames && scrubber_frames < end_frames {
        let scrubber_pixel_pos =
            ((scrubber_frames - min_frames as f32) * time_scale - virtual_viewport.x) * pixel_scale;

        let scrubber_x = (screen_cursor.x + SIDEBAR_WIDTH + scrubber_pixel_pos).floor();
        DrawList::for_overlay().draw_line(
            (scrubber_x, screen_cursor.y),
            (scrubber_x, screen_cursor.y + available_size.y),
            (1., 1., 1.),
            1.,
        );
    }

    // Move the cursor to the top of the virtual viewport
    let top_pos = pixel_scale * -virtual_viewport.y;
    unsafe { igSetCursorPosY(igGetCursorPosY() + top_pos) };

    // Draw some extra decoration lines
    draw_list.draw_line(
        (
            screen_cursor.x + CLIP_NAME_WIDTH + 1.,
            screen_cursor.y + SCRUBBER_HEIGHT - 1. + top_pos,
        ),
        (
            screen_cursor.x + SIDEBAR_WIDTH,
            screen_cursor.y + SCRUBBER_HEIGHT - 1. + top_pos,
        ),
        (0.3, 0.3, 0.3),
        1.,
    );
    draw_list.draw_line(
        (screen_cursor.x + SIDEBAR_WIDTH, screen_cursor.y),
        (
            screen_cursor.x + SIDEBAR_WIDTH,
            screen_cursor.y + available_size.y,
        ),
        (0.3, 0.3, 0.3),
        1.,
    );

    // Draw vertical lines for all keyframe positions
    for (clip_start_frame, _, _, _, clip_animation) in clips.iter() {
        let fields = /*iter::once(&clip_animation.time_property).chain(*/
            clip_animation
                .properties
                .iter()
                .flat_map(|property| match &property.target {
                    AnimatedPropertyTarget::Joined(field) => slice::from_ref(field),
                    AnimatedPropertyTarget::Separate(fields) => fields,
                });
        //);

        for field in fields {
            let mut current_pos = *clip_start_frame as i32 + field.local_offset_frames;
            draw_keyframe_line(
                &mut draw_list,
                screen_cursor,
                available_size.y,
                current_pos as f32,
                min_frames as i32,
                start_frames,
                end_frames,
                virtual_viewport.x,
                time_scale,
                pixel_scale,
            );

            for segment in &field.segments {
                current_pos += segment.duration_frames as i32;
                draw_keyframe_line(
                    &mut draw_list,
                    screen_cursor,
                    available_size.y,
                    current_pos as f32,
                    min_frames as i32,
                    start_frames,
                    end_frames,
                    virtual_viewport.x,
                    time_scale,
                    pixel_scale,
                );
            }
        }
    }

    // Start drawing properties!
    for (index, (clip_start_frame, clip_duration, clip_name, schema, clip_animation)) in
        clips.into_iter().enumerate()
    {
        unsafe {
            igPushIDInt(index as i32);
        }
        draw_clip(
            clip_start_frame,
            clip_duration,
            current_frame >= clip_start_frame && current_frame < clip_start_frame + clip_duration,
            clip_name,
            schema,
            clip_animation,
            min_frames,
            virtual_viewport,
            time_scale,
            pixel_scale,
            available_size.x,
            clip_map,
            editor_state,
        );
        unsafe {
            igPopID();
        };
    }
}

fn draw_keyframe_line(
    draw_list: &mut DrawList,
    screen_cursor: ImVec2,
    available_height: f32,
    frame_pos: f32,
    min_frames: i32,
    start_frames: f32,
    end_frames: f32,
    min_virtual: f32,
    time_scale: f32,
    pixel_scale: f32,
) {
    if frame_pos >= start_frames && frame_pos < end_frames {
        let pixel_pos = ((frame_pos - min_frames as f32) * time_scale - min_virtual) * pixel_scale;
        let pixel_x = (screen_cursor.x + SIDEBAR_WIDTH + pixel_pos).floor();
        draw_list.draw_line(
            (pixel_x, screen_cursor.y),
            (pixel_x, screen_cursor.y + available_height),
            (0.4, 0.4, 0.4),
            1.,
        );
    }
}

fn get_clip_virtual_height(clip: &AnimationClip) -> f32 {
    clip.properties
        .iter()
        .map(|prop| {
            if prop.is_collapsed {
                KEYFRAME_BAR_VIRTUAL_HEIGHT
            } else {
                KEYFRAME_BAR_VIRTUAL_HEIGHT + MOTION_BAR_VIRTUAL_HEIGHT
            }
        })
        .sum::<f32>()
    /*+ if clip.is_time_collapsed {
        KEYFRAME_BAR_VIRTUAL_HEIGHT
    } else {
        KEYFRAME_BAR_VIRTUAL_HEIGHT + MOTION_BAR_VIRTUAL_HEIGHT
    }*/
}

fn draw_clip(
    clip_start_frame: u32,
    clip_duration: u32,
    is_clip_active: bool,
    clip_name: &str,
    clip_schema: &GeneratorSchema,
    clip_animation: &mut AnimationClip,
    min_frames: u32,
    virtual_viewport: ImVec4,
    time_scale: f32,
    pixel_scale: f32,
    available_pixel_width: f32,
    clip_map: &ActiveClipMap,
    editor_state: &mut EditorState,
) {
    let clip_props_height = get_clip_virtual_height(clip_animation);
    let pixel_height = (clip_props_height * pixel_scale).floor();
    let window_y = unsafe { igGetWindowPos_nonUDT2().y } + 20.;

    let cursor_pos = unsafe { igGetCursorPos_nonUDT2() };
    if unsafe {
        igBeginChild(
            cstr!("clip"),
            ImVec2::new(available_pixel_width, pixel_height),
            false,
            ImGuiWindowFlags::NoInputs | ImGuiWindowFlags::NoScrollbar,
        )
    } {
        let mut draw_list = DrawList::for_current_window();
        let screen_cursor = unsafe { igGetCursorScreenPos_nonUDT2() };

        // draw the name of the clip on the left
        let target_name = if is_clip_active {
            match clip_map.get_clip_index(clip_animation.target_clip) {
                Some(index) => &clip_map.active_clips()[index].name,
                None => "BAD REFERENCE",
            }
        } else {
            "NOT ACTIVE"
        };
        let label = format!("{} (-> {})", clip_name, target_name);

        let text_height = calc_text_width(&label);
        let clip_name_y = screen_cursor
            .y
            .max(window_y)
            .min(screen_cursor.y + pixel_height - text_height - 10.)
            + 5.;
        draw_list.draw_vertical_text(
            (screen_cursor.x + 3., clip_name_y + text_height),
            (1., 1., 1.),
            &label,
        );

        // draw a background behind the property editor for the range our clip is visible in
        let clip_start_pixel_pos = ((clip_start_frame - min_frames) as f32 * time_scale
            - virtual_viewport.x)
            * pixel_scale;
        let clip_end_pixel_pos =
            clip_start_pixel_pos + clip_duration as f32 * time_scale * pixel_scale;
        if clip_end_pixel_pos > 1. {
            draw_list
                .rect(
                    (
                        (screen_cursor.x + SIDEBAR_WIDTH + clip_start_pixel_pos.max(1.)).floor(),
                        screen_cursor.y,
                    ),
                    (
                        (screen_cursor.x + SIDEBAR_WIDTH + clip_end_pixel_pos).floor(),
                        screen_cursor.y + pixel_height,
                    ),
                )
                .fill((1., 1., 1., 0.1))
                .draw();
        }

        // draw each property
        /*unsafe {
            igPushIDInt(0);
        }
        draw_clip_property(
            clip_start_frame,
            "Time",
            &mut clip_animation.is_time_collapsed,
            slice::from_mut(&mut clip_animation.time_property),
            Some((0., 1.)),
            min_frames,
            virtual_viewport,
            time_scale,
            pixel_scale,
            available_pixel_width,
            editor_state,
        );
        unsafe {
            igPopID();
        };*/
        let mut interaction = PropertyInteraction::None;
        for (prop_index, animated_property) in clip_animation.properties.iter_mut().enumerate() {
            let prop_fields = match &mut animated_property.target {
                AnimatedPropertyTarget::Joined(field) => slice::from_mut(field),
                AnimatedPropertyTarget::Separate(fields) => fields,
            };
            let prop_min_max = clip_schema.groups[animated_property.group_index].properties
                [animated_property.property_index]
                .value_type
                .value_range();

            unsafe { igPushIDInt(prop_index as i32 + 1) };
            interaction = interaction.union(draw_clip_property(
                prop_index,
                clip_start_frame,
                &clip_schema.groups[animated_property.group_index].properties
                    [animated_property.property_index]
                    .name,
                &mut animated_property.is_collapsed,
                prop_fields,
                prop_min_max,
                min_frames,
                virtual_viewport,
                time_scale,
                pixel_scale,
                available_pixel_width,
                editor_state,
            ));
            unsafe {
                igPopID();
            };
        }

        match interaction {
            PropertyInteraction::None => {}
            PropertyInteraction::Delete(index) => {
                clip_animation.properties.remove(index);
            }
        }
    }
    unsafe {
        igEndChild();
    };
    unsafe { igSetCursorPos(ImVec2::new(cursor_pos.x, cursor_pos.y + pixel_height)) };
}

fn get_all_keyframe_positions<'fields>(
    fields: &'fields [AnimatedPropertyField],
    clip_start_frame: i32,
) -> impl Iterator<Item = i32> + 'fields {
    fields
        .iter()
        .flat_map(|field| {
            iter::once(field.local_offset_frames).chain(field.segments.iter().scan(
                field.local_offset_frames,
                |last_end_frame, segment| {
                    let segment_end = *last_end_frame + segment.duration_frames as i32;
                    *last_end_frame = segment_end;
                    Some(segment_end)
                },
            ))
        })
        .map(move |frame| clip_start_frame + frame)
}

#[derive(Clone, Copy)]
enum PropertyInteraction {
    None,
    Delete(usize),
}

impl PropertyInteraction {
    pub fn union(self, other: PropertyInteraction) -> Self {
        match self {
            PropertyInteraction::None => other,
            _ => self,
        }
    }
}

fn draw_clip_property(
    index: usize,
    clip_start_frame: u32,
    prop_name: &str,
    is_collapsed: &mut bool,
    fields: &mut [AnimatedPropertyField],
    min_max: Option<(f32, f32)>,
    min_frames: u32,
    virtual_viewport: ImVec4,
    time_scale: f32,
    pixel_scale: f32,
    available_pixel_width: f32,
    editor_state: &mut EditorState,
) -> PropertyInteraction {
    let mut interaction = PropertyInteraction::None;

    let initial_cursor_pos = unsafe { igGetCursorPos_nonUDT2() };
    let property_height = (if *is_collapsed {
        KEYFRAME_BAR_VIRTUAL_HEIGHT
    } else {
        KEYFRAME_BAR_VIRTUAL_HEIGHT + MOTION_BAR_VIRTUAL_HEIGHT
    } * pixel_scale)
        .floor();

    if unsafe {
        igBeginChild(
            cstr!("property"),
            ImVec2::new(available_pixel_width, property_height),
            false,
            ImGuiWindowFlags::NoScrollbar,
        )
    } {
        let mut draw_list = DrawList::for_current_window();
        let screen_cursor = unsafe { igGetCursorScreenPos_nonUDT2() };
        let keyframe_bar_height = (KEYFRAME_BAR_VIRTUAL_HEIGHT * pixel_scale).floor();

        draw_list.draw_line(
            (
                screen_cursor.x + CLIP_NAME_WIDTH + 1.,
                screen_cursor.y + property_height - 1.,
            ),
            (
                screen_cursor.x + available_pixel_width,
                screen_cursor.y + property_height - 1.,
            ),
            (0.3, 0.3, 0.3),
            1.,
        );

        let prop_name_width = calc_text_width(prop_name);
        let calculated_min_max = if *is_collapsed {
            draw_list.draw_text(
                (
                    screen_cursor.x + SIDEBAR_WIDTH - prop_name_width - 5.,
                    screen_cursor.y + property_height / 2. - 7.,
                ),
                (1., 1., 1.),
                prop_name,
            );

            None
        } else {
            let (min_val, max_val) = if let Some(min_max) = min_max {
                min_max
            } else {
                fields
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(mut min_val, mut max_val), field| {
                        for component in field.start_value.fields() {
                            min_val = min_val.min(component);
                            max_val = max_val.max(component);
                        }
                        for curve_segment in &field.segments {
                            for component in curve_segment.end_value.fields() {
                                min_val = min_val.min(component);
                                max_val = max_val.max(component);
                            }
                        }

                        (min_val, max_val)
                    })
            };

            let min_str = format!("{:.2}", min_val);
            let max_str = format!("{:.2}", max_val);
            let min_str_size = calc_text_width(&min_str);
            let max_str_size = calc_text_width(&max_str);
            draw_list.draw_text(
                (
                    screen_cursor.x + SIDEBAR_WIDTH - prop_name_width - 5.,
                    screen_cursor.y
                        + keyframe_bar_height
                        + (property_height - keyframe_bar_height) / 2.
                        - 7.,
                ),
                (1., 1., 1.),
                prop_name,
            );
            draw_list.draw_text(
                (
                    screen_cursor.x + SIDEBAR_WIDTH - max_str_size - 5.,
                    screen_cursor.y + keyframe_bar_height + 2.,
                ),
                (1., 1., 1.),
                &max_str,
            );
            draw_list.draw_text(
                (
                    screen_cursor.x + SIDEBAR_WIDTH - min_str_size - 5.,
                    screen_cursor.y + property_height - 17.,
                ),
                (1., 1., 1.),
                &min_str,
            );

            Some((min_val, max_val))
        };

        // Draw the minimize/maximize button
        unsafe {
            igSetCursorScreenPos(ImVec2::new(
                screen_cursor.x + CLIP_NAME_WIDTH + 5.,
                screen_cursor.y + 6.,
            ));
        }
        let arrow_direction = if *is_collapsed {
            ImGuiDir::Down
        } else {
            ImGuiDir::Right
        };
        if unsafe { igArrowButton(cstr!("collapse"), arrow_direction) } {
            *is_collapsed = !*is_collapsed;
        }

        // Draw the delete and seek left/right buttons
        unsafe {
            igSetCursorScreenPos(ImVec2::new(
                screen_cursor.x + SIDEBAR_WIDTH - 10. - 20. * 3.,
                screen_cursor.y + 6.,
            ));
        }
        if unsafe { igArrowButton(cstr!("left seek"), ImGuiDir::Left) } {
            let seeker_pos = editor_state.current_frame() as i32;

            let all_keyframe_positions =
                get_all_keyframe_positions(fields, clip_start_frame as i32);
            let before_seeker_positions =
                all_keyframe_positions.filter(|&frame| frame < seeker_pos);
            let last_pos: Option<i32> =
                before_seeker_positions.fold(None, |last_pos, new_pos| match last_pos {
                    Some(last_pos) => Some(last_pos.max(new_pos)),
                    None => Some(new_pos),
                });
            if let Some(last_pos) = last_pos {
                editor_state.seek_to_frame(last_pos as u32);
            }
        }
        unsafe { igSameLine(0., 5.) };
        if unsafe { igArrowButton(cstr!("right seek"), ImGuiDir::Right) } {
            let seeker_pos = editor_state.current_frame() as i32;

            // Find the next keyframe position, if there is one
            let all_keyframe_positions =
                get_all_keyframe_positions(fields, clip_start_frame as i32);
            let after_seeker_positions = all_keyframe_positions.filter(|&frame| frame > seeker_pos);
            let first_pos: Option<i32> =
                after_seeker_positions.fold(None, |first_pos, new_pos| match first_pos {
                    Some(first_pos) => Some(first_pos.min(new_pos)),
                    None => Some(new_pos),
                });
            if let Some(first_pos) = first_pos {
                editor_state.seek_to_frame(first_pos as u32);
            }
        }
        unsafe { igSameLine(0., 5.) };
        if unsafe { igButton(cstr!("X"), ImVec2::new(15., 0.)) } {
            interaction = PropertyInteraction::Delete(index);
        }

        let clip_x_screen_pos = ((clip_start_frame - min_frames) as f32 * time_scale
            - virtual_viewport.x)
            * pixel_scale;
        let field_pos = ImVec2::new(
            screen_cursor.x + SIDEBAR_WIDTH + clip_x_screen_pos,
            screen_cursor.y,
        );

        // Prevent fields overlapping the sidebar
        unsafe {
            igPushClipRect(
                ImVec2::new(screen_cursor.x + SIDEBAR_WIDTH + 1., screen_cursor.y),
                ImVec2::new(
                    screen_cursor.x + available_pixel_width,
                    screen_cursor.y + property_height,
                ),
                true,
            );
        }

        if let Some((min_val, max_val)) = calculated_min_max {
            for (field_index, field) in fields.into_iter().enumerate() {
                unsafe { igPushIDInt(field_index as i32) };
                draw_field(
                    field,
                    ImVec2::new(field_pos.x, field_pos.y + keyframe_bar_height),
                    property_height - keyframe_bar_height,
                    time_scale * pixel_scale,
                    min_val - (max_val - min_val) * 0.1,
                    max_val + (max_val - min_val) * 0.1,
                    min_max,
                    field_index,
                );
                unsafe { igPopID() };
            }
        }

        // Draw a background for the keyframe bar
        draw_list
            .rect(
                (screen_cursor.x + SIDEBAR_WIDTH + 1., screen_cursor.y),
                (
                    screen_cursor.x + available_pixel_width,
                    screen_cursor.y + keyframe_bar_height,
                ),
            )
            .fill((1., 1., 1., 0.1))
            .draw();
        for (field_index, field) in fields.into_iter().enumerate() {
            unsafe { igPushIDInt(field_index as i32) };
            draw_clip_keyframe_bar(
                field,
                time_scale * pixel_scale,
                ImVec2::new(field_pos.x, field_pos.y + keyframe_bar_height / 2.),
                editor_state,
            );
            unsafe { igPopID() };
        }

        unsafe {
            igPopClipRect();
        }
    }
    unsafe {
        igEndChild();
        igSetCursorPos(ImVec2::new(
            initial_cursor_pos.x,
            initial_cursor_pos.y + property_height,
        ));
    }

    return interaction;
}

fn draw_clip_keyframe_bar(
    field: &mut AnimatedPropertyField,
    scale: f32,
    screen_pos: ImVec2,
    editor_state: &mut EditorState,
) {
    let mut interaction = KeyframeInteraction::None;

    // Draw each of the controllers
    let mut current_x = screen_pos.x + field.local_offset_frames as f32 * scale;
    interaction = interaction.union(draw_keyframe_controller(
        None,
        &mut field.local_offset_frames,
        field
            .segments
            .first_mut()
            .map(|segment| &mut segment.duration_frames),
        None,
        false,
        scale,
        ImVec2::new(current_x, screen_pos.y),
        editor_state,
    ));
    for segment_index in 0..field.segments.len() {
        unsafe { igPushIDInt(segment_index as i32) };

        let (before_segments, after_segments) = field.segments.split_at_mut(segment_index + 1);
        let last_segment = before_segments.last_mut().unwrap();
        let next_segment = after_segments.first_mut();

        let mut last_duration_int = last_segment.duration_frames as i32;
        interaction = interaction.union(draw_keyframe_controller(
            Some(segment_index),
            &mut last_duration_int,
            next_segment.map(|segment| &mut segment.duration_frames),
            Some(&mut last_segment.interpolation),
            true,
            scale,
            ImVec2::new(
                current_x + last_segment.duration_frames as f32 * scale,
                screen_pos.y,
            ),
            editor_state,
        ));
        last_segment.duration_frames = last_duration_int as u32;
        current_x += last_segment.duration_frames as f32 * scale;

        unsafe { igPopID() };
    }

    match interaction {
        KeyframeInteraction::None => {}
        KeyframeInteraction::Deleted(delete_index) => {
            delete_keyframe(field, delete_index);
        }
    }
}

#[derive(Clone, Copy)]
enum KeyframeInteraction {
    None,
    Deleted(Option<usize>),
}

impl KeyframeInteraction {
    pub fn union(self, other: KeyframeInteraction) -> Self {
        match self {
            KeyframeInteraction::None => other,
            _ => self,
        }
    }
}

fn draw_keyframe_controller(
    index: Option<usize>,
    last_duration: &mut i32,
    mut next_duration: Option<&mut u32>,
    interpolation: Option<&mut CurveInterpolation>,
    limit_last: bool,
    scale: f32,
    screen_pos: ImVec2,
    editor_state: &mut EditorState,
) -> KeyframeInteraction {
    let mut interaction = KeyframeInteraction::None;

    unsafe {
        igSetCursorScreenPos(ImVec2::new(
            screen_pos.x - KEYFRAME_CONTROLLER_WIDTH / 2.,
            screen_pos.y - KEYFRAME_CONTROLLER_HEIGHT / 2.,
        ));
        igInvisibleButton(
            cstr!("keyframe"),
            ImVec2::new(KEYFRAME_CONTROLLER_WIDTH, KEYFRAME_CONTROLLER_HEIGHT),
        );
    }

    let mut draw_list = DrawList::for_current_window();
    let keyframe_quad = draw_list
        .quad(
            (screen_pos.x, screen_pos.y - KEYFRAME_CONTROLLER_HEIGHT / 2.),
            (screen_pos.x + KEYFRAME_CONTROLLER_WIDTH / 2., screen_pos.y),
            (screen_pos.x, screen_pos.y + KEYFRAME_CONTROLLER_HEIGHT / 2.),
            (screen_pos.x - KEYFRAME_CONTROLLER_WIDTH / 2., screen_pos.y),
        )
        .stroke((0.5, 0.5, 1.), 1.);

    if unsafe { igIsItemHovered(ImGuiHoveredFlags::empty()) || igIsItemActive() } {
        keyframe_quad.fill((0.5, 0.5, 1., 1.)).draw();
    } else {
        keyframe_quad.fill((0.5, 0.5, 1., 0.5)).draw();
    }

    if unsafe { igIsItemClicked(0) } {
        editor_state.drag_start_position = *last_duration;
    }

    if unsafe { igBeginPopupContextItem(cstr!("menu"), 1) } {
        if let Some(interpolation) = interpolation {
            if unsafe {
                igMenuItemBool(
                    cstr!("Linear"),
                    ptr::null(),
                    interpolation.is_linear(),
                    true,
                )
            } {
                *interpolation = CurveInterpolation::Linear;
            }

            if unsafe {
                igMenuItemBool(
                    cstr!("Cubic bezier"),
                    ptr::null(),
                    interpolation.is_cubic_bezier(),
                    true,
                )
            } {
                *interpolation = CurveInterpolation::CubicBezier(CubicBezier::new(
                    Vector2 { x: 0.2, y: 0. },
                    Vector2 { x: 0.8, y: 1. },
                ));
            }

            unsafe { igSeparator() };
        }

        if unsafe { igMenuItemBool(cstr!("Delete"), ptr::null(), false, true) } {
            interaction = KeyframeInteraction::Deleted(index);
        }

        unsafe { igEndPopup() };
    }

    if unsafe { igIsItemActive() && igIsMouseDragging(0, 0.) } {
        let mouse_delta = unsafe { igGetMouseDragDelta_nonUDT2(0, 0.) };
        let last_last_duration = *last_duration;
        let shifted_frames = (mouse_delta.x / scale) as i32;
        let shifted_frames = if limit_last {
            (editor_state.drag_start_position + shifted_frames).max(1)
                - editor_state.drag_start_position
        } else {
            shifted_frames
        };
        let shifted_frames = if let Some(next_duration) = &next_duration {
            (editor_state.drag_start_position + shifted_frames)
                .min(last_last_duration + **next_duration as i32 - 1)
                - editor_state.drag_start_position
        } else {
            shifted_frames
        };

        *last_duration = editor_state.drag_start_position + shifted_frames;
        let duration_delta = *last_duration - last_last_duration;

        if let Some(next_duration) = &mut next_duration {
            **next_duration = (**next_duration as i32 - duration_delta) as u32;
        }
    }

    return interaction;
}

fn draw_field(
    field: &mut AnimatedPropertyField,
    screen_pos: ImVec2,
    screen_height: f32,
    scale: f32,
    min_val: f32,
    max_val: f32,
    force_min_max: Option<(f32, f32)>,
    field_index: usize,
) {
    // Adjust the position based on the local offset
    let local_offset_pixels = field.local_offset_frames as f32 * scale;
    let start_x_pos = screen_pos.x + local_offset_pixels;
    let mut current_screen_pos = ImVec2::new(start_x_pos, screen_pos.y);

    // Draw each of the curve segments
    let mut start_val = field.start_value;
    for (segment_index, segment) in field.segments.iter_mut().enumerate() {
        let segment_rect = ImVec4::new(
            current_screen_pos.x,
            current_screen_pos.y,
            current_screen_pos.x + segment.duration_frames as f32 * scale,
            current_screen_pos.y + screen_height,
        );

        // Don't bother drawing the curve if it's not visible at the moment
        if unsafe {
            igIsRectVisibleVec2(
                ImVec2::new(segment_rect.x, segment_rect.y),
                ImVec2::new(segment_rect.z, segment_rect.w),
            )
        } {
            unsafe {
                igPushIDInt(segment_index as i32);
            }
            draw_curve_segment(
                start_val,
                segment,
                segment_rect,
                min_val,
                max_val,
                field_index,
            );
            unsafe {
                igPopID();
            };
        }

        start_val = segment.end_value;
        current_screen_pos.x = segment_rect.z;
    }

    // Draw the curve controllers
    draw_curve_controller(
        &mut field.start_value,
        ImVec2::new(start_x_pos, screen_pos.y),
        screen_height,
        min_val,
        max_val,
        force_min_max,
        field_index,
    );
    let mut current_x = start_x_pos;
    for segment_index in 0..field.segments.len() {
        unsafe { igPushIDInt(segment_index as i32) };

        let segment = &mut field.segments[segment_index];
        current_x += segment.duration_frames as f32 * scale;
        draw_curve_controller(
            &mut segment.end_value,
            ImVec2::new(current_x, screen_pos.y),
            screen_height,
            min_val,
            max_val,
            force_min_max,
            field_index,
        );

        unsafe { igPopID() };
    }
}

static CURVE_FIELD_COLORS: [(f32, f32, f32); 4] =
    [(1., 0., 0.), (0., 1., 0.), (0., 0., 1.), (1., 1., 0.)];

fn get_colors_iter(field_index: usize) -> impl Iterator<Item = (f32, f32, f32)> {
    CURVE_FIELD_COLORS.iter().cycle().skip(field_index).cloned()
}

fn draw_curve_controller(
    value: &mut PropertyValue,
    screen_pos: ImVec2,
    screen_height: f32,
    min_val: f32,
    max_val: f32,
    force_min_max: Option<(f32, f32)>,
    field_index: usize,
) {
    let mut draw_list = DrawList::for_current_window();

    let mut did_change_value = false;
    let mut new_value_fields = value
        .fields()
        .zip(get_colors_iter(field_index))
        .enumerate()
        .map(|(field_index, (field_val, field_color))| {
            let normalized_val = (field_val - min_val) / (max_val - min_val);
            let point_screen_y = screen_pos.y + screen_height * (1. - normalized_val);

            unsafe {
                igPushIDInt(field_index as i32);
                igSetCursorScreenPos(ImVec2::new(screen_pos.x - 3., point_screen_y - 3.));
                igInvisibleButton(cstr!("controller"), ImVec2::new(6., 6.));
            };
            let controller_circle = draw_list
                .circle((screen_pos.x, point_screen_y), 3.)
                .stroke(field_color, 1.);
            if unsafe { igIsItemHovered(ImGuiHoveredFlags::empty()) } {
                controller_circle.fill(field_color).draw();
            } else {
                controller_circle.draw();
            }

            let new_val = if unsafe { igIsItemActive() && igIsMouseDragging(0, 0.) } {
                did_change_value = true;

                let new_screen_y = unsafe { igGetMousePos_nonUDT2().y };
                let new_val = (1. - (new_screen_y - screen_pos.y) / screen_height)
                    * (max_val - min_val)
                    + min_val;
                let new_val = if let Some((force_min, force_max)) = force_min_max {
                    new_val.max(force_min).min(force_max)
                } else {
                    new_val
                };
                unsafe {
                    let mouse_pos = igGetMousePos_nonUDT2();
                    igSetNextWindowPos(
                        ImVec2::new(mouse_pos.x, mouse_pos.y - 40.),
                        ImGuiCond::Always,
                        ImVec2::new(0., 0.),
                    );
                    igSetTooltip(cstr!("%.2f"), new_val as f64);
                }

                new_val
            } else {
                field_val
            };

            unsafe {
                igPopID();
            }

            new_val
        });
    let new_val = PropertyValue::from_fields(value.get_type(), &mut new_value_fields).unwrap();
    if did_change_value {
        *value = new_val;
    }
}

fn draw_curve_segment(
    start_value: PropertyValue,
    segment: &mut CurveSegment,
    target_rect: ImVec4,
    min_val: f32,
    max_val: f32,
    field_index: usize,
) {
    let mut draw_list = DrawList::for_current_window();

    // Calculate the interpolation points first
    let rect_width = target_rect.z - target_rect.x;
    let rect_height = target_rect.w - target_rect.y;

    let spaced_point_count = (rect_width / CURVE_PIXEL_SPACE - 0.5) as i32;
    let curve_interp_points = (1..=spaced_point_count)
        .into_iter()
        .map(|point_index| point_index as f32 * CURVE_PIXEL_SPACE / rect_width)
        .chain(iter::once(1.))
        .map(|point_x| (point_x, segment.interpolation.eval(point_x)));

    // Draw lines for each field in the PropertyValue
    let mut last_interp_x = 0.;
    let mut last_val = start_value;
    for (interp_x, interp_y) in curve_interp_points {
        let new_val = start_value.lerp(segment.end_value, interp_y).unwrap();
        let colors_iter = get_colors_iter(field_index);

        for ((this_field_val, last_field_val), line_color) in
            new_val.fields().zip(last_val.fields()).zip(colors_iter)
        {
            let last_normal_y = (last_field_val - min_val) / (max_val - min_val);
            let this_normal_y = (this_field_val - min_val) / (max_val - min_val);

            draw_list.draw_line(
                (
                    target_rect.x + last_interp_x * rect_width,
                    target_rect.y + (1. - last_normal_y) * (rect_height - 1.),
                ),
                (
                    target_rect.x + interp_x * rect_width,
                    target_rect.y + (1. - this_normal_y) * (rect_height - 1.),
                ),
                line_color,
                1.,
            );
        }

        last_interp_x = interp_x;
        last_val = new_val;
    }

    // Draw interpolation-specific controls
    match &mut segment.interpolation {
        CurveInterpolation::Linear => {}
        CurveInterpolation::CubicBezier(bezier) => {
            let c1 = bezier.c1();
            let c2 = bezier.c2();

            let c1_end = ImVec2::new(
                target_rect.x + c1.x * rect_width,
                target_rect.y + rect_height / 2. - c1.y * 0.2 * rect_height,
            );
            let c2_end = ImVec2::new(
                target_rect.x + c2.x * rect_width,
                target_rect.y + rect_height / 2. - (c2.y - 1.) * 0.2 * rect_height,
            );
            draw_list.draw_line(
                (target_rect.x, target_rect.y + rect_height / 2.),
                c1_end,
                (1., 1., 1.),
                1.,
            );
            draw_list.draw_line(
                (target_rect.x + rect_width, target_rect.y + rect_height / 2.),
                c2_end,
                (1., 1., 1.),
                1.,
            );

            unsafe {
                igSetCursorScreenPos(ImVec2::new(c1_end.x - 3., c1_end.y - 3.));
                igInvisibleButton(cstr!("c1"), ImVec2::new(6., 6.));
            };
            let c1_circle = draw_list.circle(c1_end, 3.).stroke((1., 1., 1.), 1.);
            if unsafe { igIsItemHovered(ImGuiHoveredFlags::empty()) } {
                c1_circle.fill((1., 1., 1.)).draw();
            } else {
                c1_circle.draw();
            }
            if unsafe { igIsItemActive() && igIsMouseDragging(0, 0.) } {
                let mouse_pos = unsafe { igGetMousePos_nonUDT2() };

                bezier.set_c1(Vector2 {
                    x: ((mouse_pos.x - target_rect.x) / rect_width).max(0.).min(1.),
                    y: ((target_rect.y + rect_height / 2. - mouse_pos.y) / rect_height)
                        .max(-0.5)
                        .min(0.5)
                        / 0.2,
                });
            }

            unsafe {
                igSetCursorScreenPos(ImVec2::new(c2_end.x - 3., c2_end.y - 3.));
                igInvisibleButton(cstr!("c2"), ImVec2::new(6., 6.));
            }
            let c2_circle = draw_list.circle(c2_end, 3.).stroke((1., 1., 1.), 1.);
            if unsafe { igIsItemHovered(ImGuiHoveredFlags::empty()) } {
                c2_circle.fill((1., 1., 1.)).draw();
            } else {
                c2_circle.draw();
            }
            if unsafe { igIsItemActive() && igIsMouseDragging(0, 0.) } {
                let mouse_pos = unsafe { igGetMousePos_nonUDT2() };

                bezier.set_c2(Vector2 {
                    x: ((mouse_pos.x - target_rect.x) / rect_width).min(1.).max(0.),
                    y: ((target_rect.y + rect_height / 2. - mouse_pos.y) / rect_height)
                        .max(-0.5)
                        .min(0.5)
                        / 0.2
                        + 1.,
                });
            }
        }
    }
}
