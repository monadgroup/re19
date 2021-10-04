use super::{draw_time_bar, get_fpb, zoom_to_time_scale};
use crate::cstr;
use crate::editor_state::EditorState;
use crate::imgui::{DrawList, ImColor};
use crate::timeline_interactions::{
    can_fit_clip, change_selected_clip_tracks, deselect_all_clips, get_snapping_points,
    insert_clip, move_selected_clips, remove_clip, resize_selected_clips_left,
    resize_selected_clips_right, select_clip, snap_offset, trim_empty_tracks,
};
use engine::animation::animation_clip::{
    AnimatedProperty, AnimatedPropertyField, AnimatedPropertyTarget, AnimationClip,
};
use engine::animation::clip::ClipReference;
use engine::animation::schema::GeneratorSchema;
use engine::animation::timeline::{Clip, ClipSource, Timeline, Track};
use engine::creation_context::CreationContext;
use engine::generator::GENERATOR_SCHEMAS;
use imgui_sys::{
    igBegin, igBeginChild, igBeginPopupContextItem, igButton, igCalcTextSize_nonUDT2, igDummy,
    igEnd, igEndChild, igEndPopup, igGetContentRegionAvail_nonUDT2, igGetCursorPosX,
    igGetCursorPosY, igGetCursorPos_nonUDT2, igGetCursorScreenPos_nonUDT2, igGetIO,
    igGetMouseDragDelta_nonUDT2, igGetMousePos_nonUDT2, igGetScrollX, igGetScrollY,
    igGetWindowContentRegionMax_nonUDT2, igGetWindowContentRegionWidth, igGetWindowPos_nonUDT2,
    igIndent, igInputText, igInvisibleButton, igIsItemActive, igIsItemClicked, igIsItemHovered,
    igIsKeyDown, igIsKeyPressed, igIsMouseClicked, igIsMouseDragging, igIsMouseReleased,
    igIsWindowFocused, igIsWindowHovered, igMenuItemBool, igPopID, igPopItemWidth, igPopStyleColor,
    igPopStyleVar, igPushIDInt, igPushItemWidth, igPushStyleColor, igPushStyleVarVec2,
    igResetMouseDragDelta, igSameLine, igSetCursorPosX, igSetCursorPosY, igSetKeyboardFocusHere,
    igSetMouseCursor, igSetScrollX, igSetScrollY, igSliderFloat, igText, ImGuiCol,
    ImGuiFocusedFlags, ImGuiHoveredFlags, ImGuiInputTextFlags, ImGuiMouseCursor, ImGuiStyleVar,
    ImGuiWindowFlags, ImVec2,
};
use std::ffi::CString;
use std::{mem, ptr};
use winapi::um::winuser::{VK_DELETE, VK_LEFT, VK_RIGHT, VK_SHIFT, VK_SPACE};

pub const TRACK_HEIGHT: f32 = 20.;
const TOOL_AREA_HEIGHT: f32 = 30.;

pub fn get_clip_border_color(clip: &Clip) -> ImColor {
    match clip.source {
        ClipSource::Generator(_) => (0.067, 0.298, 0.165).into(),
        ClipSource::Animation(_) => (0.3, 0.3, 0.3).into(),
    }
}

pub fn get_clip_selected_color(clip: &Clip) -> ImColor {
    match clip.source {
        ClipSource::Generator(_) => (0.227, 1.000, 0.549).into(),
        ClipSource::Animation(_) => (0.8, 0.8, 0.8).into(),
    }
}

pub fn get_clip_nonselected_color(clip: &Clip) -> ImColor {
    match clip.source {
        ClipSource::Generator(_) => (0.180, 0.800, 0.443).into(),
        ClipSource::Animation(_) => (0.6, 0.6, 0.6).into(),
    }
}

pub fn draw_timeline(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    creation_context: &mut CreationContext,
) {
    // update playback state from key presses
    if unsafe { igIsKeyPressed(VK_SPACE, true) } {
        editor_state.play_pause();
    }

    let seek_beats = if unsafe { igIsKeyDown(VK_SHIFT) } {
        editor_state.beats_per_bar
    } else {
        1
    };
    let seek_frames = editor_state.seconds_to_frame(seek_beats as f32 / editor_state.bpm * 60.);

    if unsafe { igIsKeyPressed(VK_LEFT, true) } {
        editor_state.seek_relative(-(seek_frames as i32));
    }
    if unsafe { igIsKeyPressed(VK_RIGHT, true) } {
        editor_state.seek_relative(seek_frames as i32);
    }

    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));
    }
    let show_window = unsafe {
        igBegin(
            cstr!("Timeline"),
            ptr::null_mut(),
            ImGuiWindowFlags::NoScrollbar | ImGuiWindowFlags::NoScrollWithMouse,
        )
    };
    unsafe {
        igPopStyleVar(1);
    }

    let last_zoom = editor_state.timeline_zoom;

    if show_window {
        let window_size = unsafe { igGetContentRegionAvail_nonUDT2() };

        // Display the control area
        let before_control_area_cursor = unsafe { igGetCursorPosY() };
        unsafe { igPushStyleColor(ImGuiCol::ChildBg, (0.2, 0.2, 0.2, 1.).into()) };
        let show_control_area = unsafe {
            igBeginChild(
                cstr!("controls"),
                ImVec2::new(window_size.x, TOOL_AREA_HEIGHT),
                false,
                ImGuiWindowFlags::empty(),
            )
        };
        unsafe {
            igPopStyleColor(1);
        };
        if show_control_area {
            unsafe {
                igSetCursorPosY(igGetCursorPosY() + 5.);
                igIndent(5.);
                let play_pause_title = if editor_state.is_playing() {
                    cstr!("||")
                } else {
                    cstr!(">")
                };
                if igButton(play_pause_title, ImVec2::new(50., 0.)) {
                    editor_state.play_pause();
                }
                igSameLine(0., 10.);

                let current_seconds = editor_state.frame_to_seconds(editor_state.current_frame());
                let current_beats = current_seconds * editor_state.bpm / 60.;
                igText(
                    cstr!("Time: %02u:%02u.%02u (%02u:%02u.%02us)"),
                    current_beats as u32 / editor_state.beats_per_bar,
                    current_beats as u32 % editor_state.beats_per_bar,
                    (current_beats.fract() * 100.) as u32,
                    current_seconds as u32 / 60,
                    current_seconds as u32 % 60,
                    (current_seconds.fract() * 100.) as u32,
                );

                igSameLine(0., 0.);
                igSetCursorPosX(window_size.x - 315.);
                clip_selector(timeline, editor_state);

                igSameLine(0., 10.);
                igPushItemWidth(200.);
                igSliderFloat(
                    cstr!("##zoom"),
                    &mut editor_state.timeline_zoom,
                    0.,
                    1.,
                    cstr!(""),
                    1.,
                );
                igPopItemWidth();
            }
        }
        unsafe {
            igEndChild();
        }
        unsafe { igSetCursorPosY(before_control_area_cursor + TOOL_AREA_HEIGHT) };

        // Display the scrollable timeline
        if unsafe {
            igBeginChild(
                cstr!("timeline"),
                ImVec2::new(window_size.x, window_size.y - TOOL_AREA_HEIGHT),
                false,
                ImGuiWindowFlags::HorizontalScrollbar
                    | ImGuiWindowFlags::AlwaysHorizontalScrollbar
                    | ImGuiWindowFlags::NoScrollWithMouse,
            )
        } {
            let fpb = get_fpb(editor_state.fps, editor_state.bpm);
            let time_scale = zoom_to_time_scale(last_zoom, fpb);
            let current_time_pixels = editor_state.current_frame() as f32 * time_scale;
            let screen_cursor_pos = unsafe { igGetCursorScreenPos_nonUDT2() };
            let available_height = unsafe { igGetWindowContentRegionMax_nonUDT2().y };

            let did_scroll = if unsafe { igIsWindowHovered(ImGuiHoveredFlags::ChildWindows) } {
                let scroll_delta = unsafe { (*igGetIO()).mouse_wheel };

                if scroll_delta != 0. {
                    editor_state.timeline_zoom = (editor_state.timeline_zoom + scroll_delta / 50.)
                        .max(0.)
                        .min(1.);
                    let new_time_scale = zoom_to_time_scale(editor_state.timeline_zoom, fpb);

                    let mouse_x_pixels = unsafe { igGetMousePos_nonUDT2().x - screen_cursor_pos.x };
                    let old_mouse_x_frames = mouse_x_pixels / time_scale;
                    let new_mouse_x_frames = mouse_x_pixels / new_time_scale;
                    let mouse_delta_pixels =
                        (old_mouse_x_frames - new_mouse_x_frames) * new_time_scale;
                    unsafe { igSetScrollX(igGetScrollX() + mouse_delta_pixels) };

                    true
                } else {
                    false
                }
            } else {
                false
            };

            // if the zoom changed due to the zoom bar, keep the scrubber at the same position
            if !did_scroll && last_zoom != editor_state.timeline_zoom {
                let new_time_scale = zoom_to_time_scale(editor_state.timeline_zoom, fpb);
                let new_time_pixels = editor_state.current_frame() as f32 * new_time_scale;

                unsafe {
                    igSetScrollX(igGetScrollX() + new_time_pixels - current_time_pixels);
                }
            }

            // If the mouse is being dragged, update the pan
            if unsafe {
                igIsWindowHovered(ImGuiHoveredFlags::ChildWindows) && igIsMouseDragging(2, 1.)
            } {
                let drag_delta = unsafe { igGetMouseDragDelta_nonUDT2(2, 1.) };

                unsafe {
                    igSetScrollX(igGetScrollX() - drag_delta.x);
                    igSetScrollY(igGetScrollY() - drag_delta.y);
                }

                unsafe { igResetMouseDragDelta(2) };
            }

            draw_scrubber_bar(time_scale, editor_state);

            let mut draw_list = DrawList::for_current_window();
            let window_pos = unsafe { igGetWindowPos_nonUDT2() };
            let start_cursor_pos = unsafe { igGetCursorPos_nonUDT2() };
            let available_width = unsafe { igGetWindowContentRegionWidth() };

            let mut clip_interaction = ClipInteraction::None;
            let track_count = timeline.tracks.len();
            for track_index in 0..track_count {
                clip_interaction = clip_interaction.and(draw_timeline_track(
                    timeline,
                    track_index,
                    editor_state,
                    time_scale,
                ));

                let end_cursor_y = unsafe { igGetCursorScreenPos_nonUDT2().y };

                // draw a line from the left to the right of the window
                draw_list.draw_line(
                    (window_pos.x + start_cursor_pos.x, end_cursor_y - 1.),
                    (
                        window_pos.x + start_cursor_pos.x + available_width,
                        end_cursor_y - 1.,
                    ),
                    (1., 1., 1., 0.2),
                    1.,
                );

                // move the cursor back to the start X coordinate
                unsafe {
                    igSetCursorPosX(start_cursor_pos.x);
                }
            }

            // place an empty space across the maximum zoomed-out size of the window
            let zoomed_in_width = (available_width / zoom_to_time_scale(0., fpb)) * time_scale;
            unsafe {
                igSetCursorPosX(zoomed_in_width);
                igDummy(ImVec2::new(0., 0.));
            }

            // draw the scrubber
            let mut overlay_draw_list = DrawList::for_overlay();
            let scrubber_screen_x = (screen_cursor_pos.x + current_time_pixels).floor();
            if scrubber_screen_x >= window_pos.x
                && scrubber_screen_x < window_pos.x + available_width
            {
                overlay_draw_list.draw_line(
                    (scrubber_screen_x, screen_cursor_pos.y),
                    (scrubber_screen_x, screen_cursor_pos.y + available_height),
                    (1., 1., 0.),
                    1.,
                );
            }

            match clip_interaction {
                ClipInteraction::None => {}
                _ => {
                    let delta = unsafe { igGetMouseDragDelta_nonUDT2(0, 1.) };
                    let delta_frames = (delta.x / time_scale) as i32;
                    let snap_threshold_frames = (10. / time_scale) as i32;

                    let min_duration_frames = (20. / time_scale).ceil() as u32;

                    let snapped_delta = snap_offset(
                        delta_frames,
                        &editor_state.snapping_points,
                        snap_threshold_frames,
                    );

                    match clip_interaction {
                        ClipInteraction::Moving => {
                            move_selected_clips(timeline, editor_state, snapped_delta);

                            let track_delta = ((editor_state.track_pixel_offset + delta.y)
                                / TRACK_HEIGHT)
                                .floor() as i32;
                            change_selected_clip_tracks(timeline, editor_state, track_delta);
                        }
                        ClipInteraction::ResizingLeft => {
                            resize_selected_clips_left(
                                timeline,
                                editor_state,
                                snapped_delta,
                                min_duration_frames,
                            );
                        }
                        ClipInteraction::ResizingRight => {
                            resize_selected_clips_right(
                                timeline,
                                editor_state,
                                snapped_delta,
                                min_duration_frames,
                            );
                        }
                        ClipInteraction::None => unreachable!(),
                    }
                }
            }

            let is_clicking_window = unsafe {
                igIsWindowHovered(ImGuiHoveredFlags::empty()) && igIsMouseClicked(0, false)
            };

            if is_clicking_window {
                deselect_all_clips(timeline);
                try_create_clip(
                    screen_cursor_pos,
                    time_scale,
                    timeline,
                    editor_state,
                    creation_context,
                );
            }
        }

        unsafe {
            igEndChild();
        };

        // Delete any selected clips
        if unsafe {
            igIsWindowFocused(ImGuiFocusedFlags::ChildWindows) && igIsKeyPressed(VK_DELETE, false)
        } {
            for track in &mut timeline.tracks {
                let mut clip_index = 0;
                while clip_index < track.clips.len() {
                    if track.clips[clip_index].is_selected {
                        remove_clip(track, clip_index);
                    } else {
                        clip_index += 1;
                    }
                }
            }
        }
    }
    unsafe { igEnd() };
}

fn try_create_clip(
    screen_cursor_pos: ImVec2,
    time_scale: f32,
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    creation_context: &mut CreationContext,
) {
    if editor_state.insert_clip_schema.is_none() && editor_state.insert_animation.is_none() {
        return;
    }

    let mouse_pos = unsafe { igGetMousePos_nonUDT2() };
    let track_index = ((mouse_pos.y - screen_cursor_pos.y) / TRACK_HEIGHT).floor() - 1.;

    if track_index < 0. || track_index >= timeline.tracks.len() as f32 {
        return;
    }

    let track_index_usize = track_index as usize;
    let start_frame = (mouse_pos.x - screen_cursor_pos.x) / time_scale;

    if start_frame < 0. {
        return;
    }

    let start_frame_u32 = start_frame as u32;
    if !can_fit_clip(
        &timeline.tracks[track_index_usize],
        start_frame_u32,
        1,
        false,
    ) {
        return;
    }

    if let Some(inserting_clip) = editor_state.insert_clip_schema {
        let mut clip =
            inserting_clip.instantiate(editor_state.next_clip_id, 0, 1, creation_context);

        if let Some(clip_properties) = &editor_state.insert_clip_properties {
            for (source_group, target_group) in
                clip_properties.iter().zip(clip.property_groups.iter_mut())
            {
                for (source_prop, target_prop) in
                    source_group.iter().zip(target_group.defaults.iter_mut())
                {
                    target_prop.value = *source_prop;
                }
            }
        }

        clip.is_selected = true;
        insert_clip(
            &mut timeline.tracks[track_index_usize],
            clip,
            start_frame_u32,
        )
        .ok()
        .unwrap();
        editor_state.insert_clip_schema = None;
        editor_state.insert_clip_properties = None;
    } else if let Some(animation_clip) = editor_state.insert_animation {
        let (target_ref, target_schema, group_index, prop_index, val, global_frame) =
            animation_clip;
        insert_clip(
            &mut timeline.tracks[track_index_usize],
            Clip {
                id: editor_state.next_clip_id,
                name: "Animation".to_string(),
                schema: target_schema,
                source: ClipSource::Animation(AnimationClip {
                    target_clip: target_ref,
                    /*is_time_collapsed: true,
                    time_property: AnimatedPropertyField {
                        local_offset_frames: 0,
                        start_value: PropertyValue::Float(0.),
                        segments: vec![CurveSegment {
                            duration_frames: 1,
                            end_value: PropertyValue::Float(1.),
                            interpolation: CurveInterpolation::Linear,
                        }],
                    },*/
                    properties: vec![AnimatedProperty {
                        group_index,
                        property_index: prop_index,
                        is_collapsed: false,
                        target: AnimatedPropertyTarget::Joined(AnimatedPropertyField {
                            local_offset_frames: global_frame as i32 - start_frame_u32 as i32,
                            start_value: val,
                            segments: Vec::new(),
                        }),
                    }],
                }),
                offset_frames: 0,
                duration_frames: 1,
                property_groups: Vec::new(),
                is_selected: true,
            },
            start_frame_u32,
        )
        .ok()
        .unwrap();
        editor_state.insert_animation = None;
    }

    editor_state.just_inserted_clip = Some(editor_state.next_clip_id);
    editor_state.next_clip_id += 1;
    trim_empty_tracks(timeline);

    start_interaction(timeline, editor_state);
}

fn clip_selector(timeline: &mut Timeline, editor_state: &mut EditorState) {
    unsafe {
        if let Some(inserting_schema) = editor_state.insert_clip_schema {
            let schema_c_str = CString::new(inserting_schema.name).unwrap();
            let text_width =
                igCalcTextSize_nonUDT2(schema_c_str.as_ptr(), ptr::null(), false, 0.).x;
            let cursor_x = igGetCursorPosX();
            igSetCursorPosX(cursor_x - text_width);
            igText(cstr!("Inserting: %s"), schema_c_str.as_ptr());
            igSetCursorPosX(cursor_x + 100.);
        } else if let Some(_inserting_animation) = editor_state.insert_animation {
            let cursor_x = igGetCursorPosX();
            igText(cstr!("Inserting animation"));
            igSetCursorPosX(cursor_x + 100.);
        } else if let Some(_) = editor_state.retarget_clip_request {
            let cursor_x = igGetCursorPosX();
            igText(cstr!("Retargetting animation"));
            igSetCursorPosX(cursor_x + 100.);
        } else {
            igButton(cstr!("Add Clip"), ImVec2::new(100., 0.));

            if igBeginPopupContextItem(cstr!("Select clip"), 0) {
                for schema in GENERATOR_SCHEMAS {
                    schema_menu_item(timeline, editor_state, schema);
                }
                igEndPopup();
            }
        }
    }
}

fn start_inserting_schema(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    schema: &'static GeneratorSchema,
) {
    // Ensure there are empty tracks at the start and end to allow inserting into them
    let insert_at_end = if let Some(last_track) = timeline.tracks.last() {
        !last_track.clips.is_empty()
    } else {
        true
    };
    if insert_at_end {
        timeline.tracks.push(Track::default());
    }

    let insert_at_start = if let Some(first_track) = timeline.tracks.first() {
        !first_track.clips.is_empty()
    } else {
        true
    };
    if insert_at_start {
        timeline.tracks.insert(0, Track::default());
    }

    editor_state.insert_clip_schema = Some(schema);
}

fn schema_menu_item(
    timeline: &mut Timeline,
    editor_state: &mut EditorState,
    schema: &'static GeneratorSchema,
) {
    let name_cstring = CString::new(schema.name).unwrap();
    if unsafe { igMenuItemBool(name_cstring.as_ptr(), ptr::null(), false, true) } {
        start_inserting_schema(timeline, editor_state, schema);
    }
}

fn draw_scrubber_bar(time_scale: f32, editor_state: &mut EditorState) {
    let window_x = unsafe { igGetWindowPos_nonUDT2().x };
    let start_cursor_pos = unsafe { igGetCursorScreenPos_nonUDT2() };
    let start_pixel = window_x - start_cursor_pos.x;
    let pixel_width = unsafe { igGetWindowContentRegionWidth() };

    let fpb = get_fpb(editor_state.fps, editor_state.bpm);
    let start_beats = start_pixel / (time_scale * fpb);
    let end_beats = (start_pixel + pixel_width) / (time_scale * fpb);

    // Move the cursor to the top left of the visible area
    unsafe { igSetCursorPosX(start_pixel) };

    draw_time_bar(fpb, start_beats, end_beats, pixel_width, 0., editor_state);

    if unsafe { igIsItemActive() } {
        let mouse_screen_pos = unsafe { igGetMousePos_nonUDT2() };
        editor_state
            .seek_to_frame(((mouse_screen_pos.x - start_cursor_pos.x) / time_scale).max(0.) as u32);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ClipInteraction {
    None,
    Moving,
    ResizingLeft,
    ResizingRight,
}

impl ClipInteraction {
    pub fn and(self, other: ClipInteraction) -> Self {
        match other {
            ClipInteraction::None => self,
            _ => other,
        }
    }
}

fn draw_timeline_track(
    timeline: &mut Timeline,
    track_index: usize,
    editor_state: &mut EditorState,
    time_scale: f32,
) -> ClipInteraction {
    let start_cursor_pos = unsafe { igGetCursorPos_nonUDT2() };

    let mut clip_interaction = ClipInteraction::None;
    let mut last_clip_end = 0;
    let clip_count = timeline.tracks[track_index].clips.len();
    for clip_index in 0..clip_count {
        unsafe {
            igPushIDInt(timeline.tracks[track_index].clips[clip_index].id as i32);
        }
        clip_interaction = clip_interaction.and(draw_timeline_clip(
            timeline,
            track_index,
            clip_index,
            editor_state,
            last_clip_end,
            time_scale,
        ));
        unsafe {
            igPopID();

            igSetCursorPosX(start_cursor_pos.x);
            igSetCursorPosY(start_cursor_pos.y);
        }

        let clip = &timeline.tracks[track_index].clips[clip_index];
        last_clip_end += clip.offset_frames + clip.duration_frames;
    }

    unsafe {
        igSetCursorPosY(start_cursor_pos.y + TRACK_HEIGHT);
    }

    clip_interaction
}

fn set_item_mouse_cursor(cursor: ImGuiMouseCursor) {
    if unsafe { igIsItemHovered(ImGuiHoveredFlags::empty()) || igIsItemActive() } {
        unsafe {
            igSetMouseCursor(cursor);
        }
    }
}

fn start_interaction(timeline: &Timeline, editor_state: &mut EditorState) {
    editor_state.drag_offset = 0;
    editor_state.track_offset = 0;
    editor_state.track_pixel_offset =
        unsafe { igGetMousePos_nonUDT2().y - igGetCursorScreenPos_nonUDT2().y };
    editor_state.snapping_points = get_snapping_points(timeline);
}

fn try_start_interaction(
    timeline: &mut Timeline,
    track_index: usize,
    clip_index: usize,
    editor_state: &mut EditorState,
) -> bool {
    if unsafe { igIsItemClicked(0) } {
        select_clip(timeline, track_index, clip_index, unsafe {
            !igIsKeyDown(VK_SHIFT)
        });
        start_interaction(timeline, editor_state);
    }

    unsafe { igIsItemActive() && igIsMouseDragging(0, 1.) }
}

fn can_set_target(editor_state: &EditorState, clip: &Clip) -> bool {
    if let Some((req_schema, _)) = editor_state.retarget_clip_request {
        clip.source.is_generator()
            && clip.schema as *const GeneratorSchema == req_schema as *const GeneratorSchema
    } else {
        false
    }
}

fn try_receive_target(editor_state: &mut EditorState, clip: &mut Clip) {
    let request_source = match editor_state.retarget_clip_request {
        Some((_, r)) => r,
        None => return,
    };
    if request_source.clip_id() != clip.id {
        return;
    }

    let new_target = match editor_state.retarget_clip_response {
        Some(t) => t,
        None => return,
    };
    let animation = match &mut clip.source {
        ClipSource::Animation(a) => a,
        ClipSource::Generator(_) => return,
    };

    animation.target_clip = new_target;
    editor_state.retarget_clip_request = None;
    editor_state.retarget_clip_response = None;
}

fn draw_timeline_clip(
    timeline: &mut Timeline,
    track_index: usize,
    clip_index: usize,
    editor_state: &mut EditorState,
    last_clip_end_frame: u32,
    time_scale: f32,
) -> ClipInteraction {
    // Move the cursor left to account for spacing between clips
    let clip = &timeline.tracks[track_index].clips[clip_index];
    let clip_schema = clip.schema;
    let clip_id = clip.id;
    let cursor_start_x = unsafe { igGetCursorPosX() }
        + ((last_clip_end_frame + clip.offset_frames) as f32 * time_scale).floor();
    unsafe {
        igSetCursorPosX(cursor_start_x);
    }

    let clip_width = ((last_clip_end_frame + clip.offset_frames + clip.duration_frames) as f32
        * time_scale
        - cursor_start_x)
        .floor();
    let cursor_start_y = unsafe { igGetCursorPosY() };
    let window_x = unsafe { igGetWindowPos_nonUDT2().x };

    // Make invisible buttons to allow interactivity
    // There are 5px wide buttons on the left and right of each clip for resizing, and one big one
    // in the middle for selecting.
    let show_resize_buttons = clip_width >= 15.;

    // Add middle select button
    let middle_offset = if show_resize_buttons { 5. } else { 0. };
    unsafe {
        igSetCursorPosX(cursor_start_x + middle_offset);
        if clip_width > 0. {
            igInvisibleButton(
                cstr!("clip_button"),
                ImVec2::new(clip_width - middle_offset * 2., TRACK_HEIGHT),
            );
        }
        igSetCursorPosX(cursor_start_x);
        igSetCursorPosY(cursor_start_y);
    };

    try_receive_target(
        editor_state,
        &mut timeline.tracks[track_index].clips[clip_index],
    );

    let mut clip_interaction = ClipInteraction::None;
    if clip_width > 0. {
        if editor_state.renaming_clip == Some(ClipReference::new(clip_id)) {
            let mut data_str = String::new();
            mem::swap(
                &mut data_str,
                &mut timeline.tracks[track_index].clips[clip_index].name,
            );
            let mut data_bytes = data_str.into_bytes();
            data_bytes.push(0);
            data_bytes.resize(data_bytes.len() + 10, 0); // reserve space for 10 more chars

            if unsafe {
                igSetKeyboardFocusHere(0);
                igPushItemWidth(clip_width);
                let is_finished = igInputText(
                    cstr!(""),
                    &mut data_bytes[0] as *mut u8 as *mut i8,
                    data_bytes.capacity(),
                    ImGuiInputTextFlags::EnterReturnsTrue | ImGuiInputTextFlags::AutoSelectAll,
                    None,
                    ptr::null_mut(),
                );
                igPopItemWidth();
                is_finished
            } {
                editor_state.renaming_clip = None;
            }

            // Remove the first null char and everything after it
            if let Some(char_index) = data_bytes.iter().position(|&byte| byte == 0) {
                data_bytes.truncate(char_index);
            }

            timeline.tracks[track_index].clips[clip_index].name =
                String::from_utf8(data_bytes).unwrap();
        } else if unsafe { igIsItemClicked(0) }
            && can_set_target(
                editor_state,
                &timeline.tracks[track_index].clips[clip_index],
            )
        {
            editor_state.retarget_clip_response = Some(ClipReference::new(clip_id));
        } else if unsafe { igBeginPopupContextItem(cstr!("menu"), 1) } {
            if unsafe { igMenuItemBool(cstr!("Rename"), ptr::null(), false, true) } {
                editor_state.renaming_clip = Some(ClipReference::new(clip_id));
            }

            let clip = &timeline.tracks[track_index].clips[clip_index];
            if clip.source.is_generator() {
                if unsafe { igMenuItemBool(cstr!("Copy"), ptr::null(), false, true) } {
                    editor_state.insert_clip_properties = Some(
                        clip.property_groups
                            .iter()
                            .map(|group| group.defaults.iter().map(|prop| prop.value).collect())
                            .collect(),
                    );
                    editor_state.insert_clip_schema = Some(clip_schema);
                }
            } else {
                if unsafe { igMenuItemBool(cstr!("Retarget"), ptr::null(), false, true) } {
                    editor_state.retarget_clip_request =
                        Some((clip.schema, ClipReference::new(clip.id)));
                }
            }

            unsafe { igEndPopup() };
        }

        let clip = &timeline.tracks[track_index].clips[clip_index];
        let is_clicking = unsafe { igIsItemClicked(0) };
        if is_clicking
            && editor_state.select_clip_request.is_some()
            && editor_state.select_clip_response.is_none()
        {
            editor_state.select_clip_response = Some(ClipReference::new(clip_id));
        } else if is_clicking
            && editor_state.insert_animation.is_some()
            && clip.source.is_animation()
        {
            let clip_start_time = last_clip_end_frame + clip.offset_frames;
            let animation_clip = timeline.tracks[track_index].clips[clip_index]
                .source
                .animation_mut()
                .unwrap();
            let (target_clip, _target_schema, group_index, prop_index, val, global_time) =
                editor_state.insert_animation.unwrap();
            if animation_clip.target_clip == target_clip {
                animation_clip.properties.push(AnimatedProperty {
                    group_index,
                    property_index: prop_index,
                    is_collapsed: false,
                    target: AnimatedPropertyTarget::Joined(AnimatedPropertyField {
                        local_offset_frames: global_time as i32 - clip_start_time as i32,
                        start_value: val,
                        segments: Vec::new(),
                    }),
                });
                editor_state.insert_animation = None;
            }
        } else if try_start_interaction(timeline, track_index, clip_index, editor_state) {
            clip_interaction = ClipInteraction::Moving;
        }
    }

    if show_resize_buttons {
        // Add left resize button
        unsafe {
            igInvisibleButton(cstr!("left_resize"), ImVec2::new(5., TRACK_HEIGHT));
        }
        set_item_mouse_cursor(ImGuiMouseCursor::ResizeEW);
        if try_start_interaction(timeline, track_index, clip_index, editor_state) {
            clip_interaction = ClipInteraction::ResizingLeft;
        }

        // Add right resize button
        unsafe {
            igSetCursorPosX(cursor_start_x + clip_width - 5.);
            igSetCursorPosY(cursor_start_y);
            igInvisibleButton(cstr!("right_resize"), ImVec2::new(5., TRACK_HEIGHT));
        }
        set_item_mouse_cursor(ImGuiMouseCursor::ResizeEW);
        if try_start_interaction(timeline, track_index, clip_index, editor_state) {
            clip_interaction = ClipInteraction::ResizingRight;
        }
    }

    if editor_state.just_inserted_clip == Some(clip_id) {
        if unsafe { igIsMouseReleased(0) } {
            editor_state.just_inserted_clip = None;
        } else {
            unsafe { igSetMouseCursor(ImGuiMouseCursor::ResizeEW) };
            clip_interaction = ClipInteraction::ResizingRight;
        }
    }

    unsafe {
        igSetCursorPosX(cursor_start_x);
        igSetCursorPosY(cursor_start_y);
    }
    if unsafe {
        igBeginChild(
            cstr!("clip"),
            ImVec2::new(clip_width, TRACK_HEIGHT),
            false,
            ImGuiWindowFlags::NoScrollbar
                | ImGuiWindowFlags::NoScrollWithMouse
                | ImGuiWindowFlags::NoBackground
                | ImGuiWindowFlags::NoInputs,
        )
    } && editor_state.renaming_clip != Some(ClipReference::new(clip_id))
    {
        let clip = &timeline.tracks[track_index].clips[clip_index];
        let mut draw_list = DrawList::for_current_window();
        let top_left = unsafe { igGetCursorScreenPos_nonUDT2() };
        let bottom_right = ImVec2::new(top_left.x + clip_width, top_left.y + TRACK_HEIGHT);

        let minor_color = get_clip_border_color(clip);
        let major_color = if clip.is_selected {
            get_clip_selected_color(clip)
        } else {
            get_clip_nonselected_color(clip)
        };

        draw_list
            .rect(top_left, (bottom_right.x - 1., bottom_right.y - 1.))
            .fill(major_color)
            .draw();

        let text_x = window_x.max(top_left.x) + 5.;
        draw_list.draw_text((text_x, top_left.y + 3.), (0., 0., 0.), &clip.name);
        draw_list.draw_line(
            (top_left.x, bottom_right.y - 1.),
            (bottom_right.x, bottom_right.y - 1.),
            minor_color,
            1.,
        );
        draw_list.draw_line(
            (bottom_right.x - 1., top_left.y),
            (bottom_right.x - 1., bottom_right.y - 1.),
            minor_color,
            1.,
        );
    }
    unsafe { igEndChild() };

    clip_interaction
}
