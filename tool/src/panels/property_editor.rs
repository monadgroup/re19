use super::timeline::{
    get_clip_border_color, get_clip_nonselected_color, get_clip_selected_color, TRACK_HEIGHT,
};
use crate::cstr;
use crate::editor_state::EditorState;
use crate::imgui::DrawList;
use crate::timeline_interactions::{deselect_all_clips, insert_keyframe, select_clip};
use engine::animation::animation_clip::AnimatedPropertyTarget;
use engine::animation::clip::{ActiveClipMap, ClipPropertyValue, ClipReference};
use engine::animation::property::PropertyValue;
use engine::animation::schema::GeneratorSchema;
use engine::animation::timeline::{Clip, ClipSource, PropertyDefault, Timeline};
use engine::math::Quaternion;
use imgui_sys::{
    igArrowButton, igBegin, igButton, igCalcTextSize_nonUDT2, igCheckbox, igColorPicker3,
    igColorPicker4, igDragFloat, igDragFloat2, igDragFloat3, igDragFloat4, igEnd,
    igGetContentRegionAvailWidth, igGetCursorPosX, igGetCursorScreenPos_nonUDT2, igGetIDStr,
    igGetMousePos_nonUDT2, igInvisibleButton, igIsItemClicked, igIsItemHovered, igIsKeyDown,
    igPopID, igPopItemWidth, igPopStyleColor, igPushIDInt, igPushItemWidth, igPushStyleColor,
    igSameLine, igSetCursorPosX, igText, igTreeNodeExPtr, igTreePop, ImGuiCol, ImGuiColorEditFlags,
    ImGuiDir, ImGuiHoveredFlags, ImGuiTreeNodeFlags, ImGuiWindowFlags, ImVec2, ImVec4,
};
use std::ffi::CString;
use std::{iter, ptr};
use winapi::um::winuser::VK_SHIFT;

pub fn draw_property_editor(
    timeline: &mut Timeline,
    clip_map: &ActiveClipMap,
    editor_state: &mut EditorState,
) {
    let active_clips = clip_map.active_clips();
    let show_window = unsafe {
        igBegin(
            cstr!("Property Editor"),
            ptr::null_mut(),
            ImGuiWindowFlags::AlwaysVerticalScrollbar,
        )
    };

    if unsafe { igButton(cstr!("Select Active"), ImVec2::new(0., 0.)) } {
        if !unsafe { igIsKeyDown(VK_SHIFT) } {
            deselect_all_clips(timeline);
        }

        // select all active clips
        for active_clip in active_clips {
            select_clip(
                timeline,
                active_clip.track_index,
                active_clip.clip_index,
                false,
            );
        }
    }

    if show_window {
        let mut clip_rects = Vec::new();
        let mut new_keyframes = Vec::new();
        let mut selected_clips = timeline
            .tracks
            .iter_mut()
            .flat_map(|track| track.clips.iter_mut())
            .filter(|clip| {
                clip.is_selected
                    && match &clip.source {
                        ClipSource::Animation(_) => false,
                        _ => true,
                    }
            });

        // If there are any selected clips, use them. Otherwise, display all active clips.
        let first_clip = selected_clips.next();
        if let Some(first_clip) = first_clip {
            for selected_clip in iter::once(first_clip).chain(selected_clips) {
                draw_clip_editor(
                    selected_clip,
                    clip_map,
                    &mut clip_rects,
                    &mut new_keyframes,
                    editor_state,
                );
            }
        } else {
            for active_clip in clip_map.active_clips() {
                let clip = match timeline.tracks[active_clip.track_index]
                    .clips
                    .get_mut(active_clip.clip_index)
                {
                    Some(clip) => clip,
                    None => continue, // the clip was deleted
                };

                draw_clip_editor(
                    clip,
                    clip_map,
                    &mut clip_rects,
                    &mut new_keyframes,
                    editor_state,
                );
            }
        }

        // Render the clip rects
        let mut draw_list = DrawList::for_current_window();
        for (clip_ref, top_left, bottom_right, is_hovered) in clip_rects.into_iter() {
            if let Some(clip_ref) = clip_ref {
                if let Some(ref_target_index) = clip_map.get_clip_index(clip_ref) {
                    let active_clip = &active_clips[ref_target_index];
                    let clip =
                        &timeline.tracks[active_clip.track_index].clips[active_clip.clip_index];

                    let minor_color = get_clip_border_color(clip);
                    let major_color = if is_hovered {
                        get_clip_selected_color(clip)
                    } else {
                        get_clip_nonselected_color(clip)
                    };

                    draw_list
                        .rect(top_left, (bottom_right.x - 1., bottom_right.y - 1.))
                        .fill(major_color)
                        .draw();
                    draw_list.draw_text(
                        (top_left.x + 5., top_left.y + 3.),
                        (0., 0., 0.),
                        &clip.name,
                    );
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
                } else {
                    draw_list
                        .rect(top_left, bottom_right)
                        .fill((1., 0., 0., if is_hovered { 0.4 } else { 0.2 }))
                        .draw();
                    draw_list.draw_text(
                        (top_left.x + 5., top_left.y + 3.),
                        (1., 0., 0.),
                        "Bad reference",
                    );
                }
            } else {
                draw_list
                    .rect(top_left, bottom_right)
                    .fill((1., 1., 1., if is_hovered { 0.4 } else { 0.2 }))
                    .draw();
                draw_list.draw_text(
                    (top_left.x + 5., top_left.y + 3.),
                    (1., 1., 1.),
                    "Empty reference",
                );
            }
        }

        // Create the new keyframes at the specified places
        for (animation_clip_ref, source_clip_ref, group_index, prop_index, value) in
            new_keyframes.into_iter()
        {
            let active_clip =
                &clip_map.active_clips()[clip_map.get_clip_index(animation_clip_ref).unwrap()];
            let animation_clip = match &mut timeline.tracks[active_clip.track_index].clips
                [active_clip.clip_index]
                .source
            {
                ClipSource::Animation(animation) => animation,
                _ => panic!("Trying to insert keyframe in non-animation clip!"),
            };

            // Find the property in the animation clip
            let prop = animation_clip
                .properties
                .iter_mut()
                .find(|prop| prop.group_index == group_index && prop.property_index == prop_index)
                .unwrap();
            let could_insert_keyframe = match &mut prop.target {
                AnimatedPropertyTarget::Joined(field) => {
                    insert_keyframe(field, active_clip.local_time as i32, value)
                }
                AnimatedPropertyTarget::Separate(fields) => fields
                    .iter_mut()
                    .all(|field| insert_keyframe(field, active_clip.local_time as i32, value)),
            };

            if could_insert_keyframe {
                // Clear the override flag on the original clip
                let source_active_clip =
                    &clip_map.active_clips()[clip_map.get_clip_index(source_clip_ref).unwrap()];
                let source_clip = &mut timeline.tracks[source_active_clip.track_index].clips
                    [source_active_clip.clip_index];
                source_clip.property_groups[group_index].defaults[prop_index].is_override = false;
            }
        }
    }
    unsafe { igEnd() };
}

fn draw_clip_editor(
    clip: &mut Clip,
    clip_map: &ActiveClipMap,
    clip_rects: &mut Vec<(Option<ClipReference>, ImVec2, ImVec2, bool)>,
    new_keyframes: &mut Vec<(ClipReference, ClipReference, usize, usize, PropertyValue)>,
    editor_state: &mut EditorState,
) {
    let active_clip = clip_map
        .get_clip_index(ClipReference::new(clip.id))
        .map(|index| &clip_map.active_clips()[index]);
    let clip_schema = clip.schema;

    let header_title = match active_clip {
        Some(_) => format!("{} ({})\0", &clip.name, &clip_schema.name),
        None => format!("{} ({}) NOT ACTIVE!\0", &clip.name, &clip_schema.name),
    };

    if unsafe {
        igTreeNodeExPtr(
            clip.id as usize as *const _,
            ImGuiTreeNodeFlags::DefaultOpen,
            cstr!("%s"),
            header_title.as_ptr() as *const _,
        )
    } {
        for (group_index, (group, group_schema)) in clip
            .property_groups
            .iter_mut()
            .zip(clip_schema.groups.iter())
            .enumerate()
        {
            let group_name_cstr = CString::new(group_schema.name).unwrap();
            if unsafe {
                igTreeNodeExPtr(
                    group_index as *const _,
                    ImGuiTreeNodeFlags::DefaultOpen,
                    cstr!("%s"),
                    group_name_cstr.as_ptr(),
                )
            } {
                for (prop_index, (prop, _prop_schema)) in group
                    .defaults
                    .iter_mut()
                    .zip(group_schema.properties.iter())
                    .enumerate()
                {
                    let active_prop = active_clip
                        .map(|active_clip| &active_clip.properties[group_index][prop_index]);
                    unsafe {
                        igPushIDInt(prop_index as i32);
                    }
                    draw_property(
                        ClipReference::new(clip.id),
                        group_index,
                        prop_index,
                        prop,
                        active_prop,
                        clip_schema,
                        clip_rects,
                        new_keyframes,
                        editor_state,
                    );
                    unsafe {
                        igPopID();
                    }
                }

                unsafe {
                    igTreePop();
                }
            }
        }

        unsafe {
            igTreePop();
        }
    }
}

fn draw_property(
    current_clip: ClipReference,
    group_index: usize,
    prop_index: usize,
    prop: &mut PropertyDefault,
    active_prop: Option<&ClipPropertyValue>,
    schema: &'static GeneratorSchema,
    clip_rects: &mut Vec<(Option<ClipReference>, ImVec2, ImVec2, bool)>,
    new_keyframes: &mut Vec<(ClipReference, ClipReference, usize, usize, PropertyValue)>,
    editor_state: &mut EditorState,
) {
    let mut current_val = match active_prop {
        Some(active_prop) => active_prop.value,
        None => prop.value,
    };

    // display/allow changing override state and keyframe button
    let is_targeted = match active_prop {
        Some(active_prop) => active_prop.targeted_by.is_some(),
        None => false,
    };
    let is_overridden = match active_prop {
        Some(active_prop) => active_prop.is_overridden,
        None => false,
    };
    let mut is_overridden_checked = is_overridden || !is_targeted;
    unsafe {
        if is_targeted {
            igPushStyleColor(ImGuiCol::FrameBg, ImVec4::new(1., 0., 0., 0.5));
            igPushStyleColor(ImGuiCol::FrameBgHovered, ImVec4::new(1., 0., 0., 0.8));
            igPushStyleColor(ImGuiCol::FrameBgActive, ImVec4::new(1., 0., 0., 1.));
        }

        if igCheckbox(cstr!("##overridden"), &mut is_overridden_checked) && is_targeted {
            prop.is_override = is_overridden_checked;
            if let Some(active_prop) = active_prop {
                prop.value = active_prop.value;
            }
        }

        if is_targeted {
            igPopStyleColor(3);
        }

        igSameLine(0., -1.);
        if igArrowButton(cstr!("##keyframe"), ImGuiDir::Down) {
            //let target_animation_clip = active_prop.and_then(|prop| prop.targeted_by);
            if let Some(target_animation_clip) = active_prop.and_then(|prop| prop.targeted_by) {
                new_keyframes.push((
                    target_animation_clip,
                    current_clip,
                    group_index,
                    prop_index,
                    current_val,
                ));
            } else {
                editor_state.insert_animation = Some((
                    current_clip,
                    schema,
                    group_index,
                    prop_index,
                    current_val,
                    editor_state.current_frame(),
                ));
            }
        }
        igSameLine(0., -1.);
    }

    // display name
    let schema_prop = &schema.groups[group_index].properties[prop_index];
    let name_cstr = CString::new(&schema_prop.name as &str).unwrap();
    unsafe {
        let current_x = igGetCursorPosX();

        let reserve_width = 90.;
        let text_width = igCalcTextSize_nonUDT2(name_cstr.as_ptr(), ptr::null(), false, 0.);

        igSetCursorPosX(current_x + reserve_width - text_width.x - 10.);
        igText(cstr!("%s"), name_cstr.as_ptr());
        igSameLine(current_x + reserve_width, -1.);
        igPushItemWidth(igGetContentRegionAvailWidth());
    };

    let did_change = match &mut current_val {
        PropertyValue::Float(val) => unsafe {
            igDragFloat(cstr!(""), val, 0.01, 0., 0., cstr!("%.3f"), 2.)
        },
        PropertyValue::Vec2(val) => unsafe {
            igDragFloat2(cstr!(""), &mut val.x, 0.01, 0., 0., cstr!("%.3f"), 2.)
        },
        PropertyValue::Vec3(val) => unsafe {
            igDragFloat3(cstr!(""), &mut val.x, 0.01, 0., 0., cstr!("%.3f"), 2.)
        },
        PropertyValue::Vec4(val) => unsafe {
            igDragFloat4(cstr!(""), &mut val.x, 0.01, 0., 0., cstr!("%.3f"), 2.)
        },
        PropertyValue::RgbColor(val) => unsafe {
            igColorPicker3(
                cstr!(""),
                &mut val.0.x,
                ImGuiColorEditFlags::RGB
                    | ImGuiColorEditFlags::Float
                    | ImGuiColorEditFlags::PickerHueWheel
                    | ImGuiColorEditFlags::HDR,
            )
        },
        PropertyValue::RgbaColor(val) => unsafe {
            igColorPicker4(
                cstr!(""),
                &mut val.0.x,
                ImGuiColorEditFlags::RGB
                    | ImGuiColorEditFlags::Float
                    | ImGuiColorEditFlags::AlphaBar
                    | ImGuiColorEditFlags::PickerHueWheel
                    | ImGuiColorEditFlags::AlphaPreview
                    | ImGuiColorEditFlags::HDR,
                ptr::null(),
            )
        },
        PropertyValue::Rotation(val) => {
            // todo: there's probably something nicer we can do to display this
            let euler = val.as_euler();
            let mut euler_degrees = (
                euler.0.to_degrees(),
                euler.1.to_degrees(),
                euler.2.to_degrees(),
            );

            if unsafe {
                igDragFloat3(
                    cstr!(""),
                    &mut euler_degrees.0,
                    1.,
                    0.,
                    0.,
                    cstr!("%.3f"),
                    1.,
                )
            } {
                *val = Quaternion::euler(
                    euler_degrees.0.to_radians(),
                    euler_degrees.1.to_radians(),
                    euler_degrees.2.to_radians(),
                );
                true
            } else {
                false
            }
        }
        PropertyValue::ClipReference(val) => {
            let available_width = unsafe { igGetContentRegionAvailWidth() };
            let top_left_pos = unsafe { igGetCursorScreenPos_nonUDT2() };
            let bottom_right_pos = ImVec2::new(
                top_left_pos.x + available_width,
                top_left_pos.y + TRACK_HEIGHT,
            );
            unsafe {
                igInvisibleButton(cstr!(""), ImVec2::new(available_width, TRACK_HEIGHT));
            }

            // Defer drawing it until later, since we can't get a reference to the clip in here
            clip_rects.push((*val, top_left_pos, bottom_right_pos, unsafe {
                igIsItemHovered(ImGuiHoveredFlags::empty())
            }));

            let ref_button_id = unsafe { igGetIDStr(cstr!("ref button")) };
            if unsafe { igIsItemClicked(0) } {
                if editor_state.select_clip_request == Some(ref_button_id) {
                    editor_state.select_clip_request = None;
                    editor_state.select_clip_response = None;
                } else {
                    editor_state.select_clip_request = Some(ref_button_id);
                    editor_state.select_clip_response = None;
                }
            }

            if editor_state.select_clip_request == Some(ref_button_id) {
                let mut overlay_draw_list = DrawList::for_overlay();
                let cursor_pos = unsafe { igGetMousePos_nonUDT2() };
                overlay_draw_list.draw_line(
                    (
                        top_left_pos.x + available_width / 2.,
                        top_left_pos.y + TRACK_HEIGHT / 2.,
                    ),
                    cursor_pos,
                    (1., 0., 0.),
                    2.,
                );

                if let Some(clip_response) = editor_state.select_clip_response {
                    *val = Some(clip_response);
                    editor_state.select_clip_request = None;
                    editor_state.select_clip_response = None;

                    true
                } else {
                    false
                }
            } else if unsafe { igIsItemClicked(1) } {
                *val = None;
                editor_state.select_clip_request = None;
                editor_state.select_clip_response = None;

                true
            } else {
                false
            }
        }
    };

    unsafe { igPopItemWidth() };

    if did_change {
        prop.value = current_val;

        // Update the override mode: if currently animating, enable override
        if let Some(active_prop) = active_prop {
            if active_prop.targeted_by.is_some() {
                prop.is_override = true;
            }
        }
    }
}
