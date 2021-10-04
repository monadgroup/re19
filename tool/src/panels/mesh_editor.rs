use crate::mesh_list::{MeshCommand, MeshDescription, MeshList};
use engine::math::Quaternion;
use engine::mesh::Mesh;
use engine::mesh_gen::commands::{EditorCommandPropertyType, COMMAND_TYPES};
use imgui_sys::{
    igBegin, igBeginChild, igBeginCombo, igBeginPopupContextItem, igButton, igColorEdit3,
    igColorEdit4, igDragFloat, igDragFloat2, igDragFloat3, igDragFloat4, igDragInt, igEnd,
    igEndChild, igEndCombo, igEndPopup, igGetContentRegionAvail_nonUDT2, igGetCursorPosY,
    igGetCursorPos_nonUDT2, igIndent, igInputText, igIsItemHovered, igIsMouseClicked,
    igIsWindowHovered, igMenuItemBool, igPopID, igPopItemWidth, igPopStyleColor, igPopStyleVar,
    igPushIDInt, igPushItemWidth, igPushStyleColor, igPushStyleVarVec2, igSameLine, igSelectable,
    igSeparator, igSetCursorPos, igSetCursorPosX, igSetCursorPosY, igSetKeyboardFocusHere, igText,
    ImGuiCol, ImGuiColorEditFlags, ImGuiComboFlags, ImGuiHoveredFlags, ImGuiInputTextFlags,
    ImGuiSelectableFlags, ImGuiStyleVar, ImGuiWindowFlags, ImVec2, ImVec4,
};
use std::ffi::CString;
use std::{mem, ptr};

const MESH_WIDTH: f32 = 200.;
const PROP_EDITOR_HEIGHT: f32 = 80.;

pub fn draw_mesh_editor(mesh_list: &mut MeshList) {
    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));
    }
    let show_window = unsafe {
        igBegin(
            cstr!("Mesh Editor"),
            ptr::null_mut(),
            ImGuiWindowFlags::empty(),
        )
    };
    unsafe {
        igPopStyleVar(1);
    }

    if show_window {
        let available_height = unsafe { igGetContentRegionAvail_nonUDT2() }.y;

        let cursor_y = unsafe { igGetCursorPosY() };
        if unsafe {
            igBeginChild(
                cstr!("mesh list"),
                ImVec2::new(0., available_height - PROP_EDITOR_HEIGHT),
                false,
                ImGuiWindowFlags::HorizontalScrollbar,
            )
        } {
            draw_mesh_list_panel(mesh_list);
        }
        unsafe {
            igEndChild();
        }

        unsafe { igSetCursorPosY(cursor_y + available_height - PROP_EDITOR_HEIGHT) };
        unsafe { igPushStyleColor(ImGuiCol::ChildBg, ImVec4::new(0.15, 0.15, 0.15, 1.)) };
        let show_child = unsafe {
            igBeginChild(
                cstr!("mesh props"),
                ImVec2::new(0., PROP_EDITOR_HEIGHT),
                false,
                ImGuiWindowFlags::HorizontalScrollbar,
            )
        };
        unsafe { igPopStyleColor(1) };
        if show_window {
            draw_mesh_prop_panel(mesh_list);
        }
        unsafe {
            igEndChild();
        }
    }

    unsafe { igEnd() };
}

fn draw_mesh_list_panel(mesh_list: &mut MeshList) {
    let mut regen_description = None;
    let mut hidden_description = None;
    let mut delete_description = None;

    for (selection_index, description_index) in
        mesh_list.selected_descriptions.iter_mut().enumerate()
    {
        unsafe { igPushIDInt(*description_index as i32) };

        let mut is_selected = mesh_list.selected_index == selection_index;
        match draw_mesh_description(
            &mut mesh_list.descriptions,
            description_index,
            &mut is_selected,
        ) {
            MeshDescriptionInteraction::None => {}
            MeshDescriptionInteraction::Hidden => {
                hidden_description = Some(*description_index);
            }
            MeshDescriptionInteraction::Deleted => {
                delete_description = Some(selection_index);
            }
            MeshDescriptionInteraction::Changed => {
                regen_description = Some(*description_index);
            }
        }
        if is_selected {
            mesh_list.selected_index = selection_index;
        }

        unsafe {
            igSameLine(0., 0.);
            igPopID();
        }
    }

    if let Some(regen_description) = regen_description {
        mesh_list.regen_mesh(regen_description);
    }
    if let Some(hidden_description) = hidden_description {
        mesh_list.selected_descriptions.remove(hidden_description);
    }
    if let Some(delete_description) = delete_description {
        mesh_list.descriptions.remove(delete_description);

        // Delete the selected descriptions, decrement those that are after so they still point to the same place
        let mut selected_index = 0;
        while selected_index < mesh_list.selected_descriptions.len() {
            if mesh_list.selected_descriptions[selected_index] == delete_description {
                mesh_list.selected_descriptions.remove(selected_index);
                continue;
            }

            if mesh_list.selected_descriptions[selected_index] > delete_description {
                mesh_list.selected_descriptions[selected_index] -= 1;
            }

            selected_index += 1;
        }
    }

    // Ensure the selected mesh is valid
    if mesh_list.selected_descriptions.is_empty() {
        mesh_list.selected_index = 0;
    } else {
        mesh_list.selected_index = mesh_list
            .selected_index
            .min(mesh_list.selected_descriptions.len() - 1);
    }

    // If there are no mesh descriptions or the last one isn't empty, make a new empty one
    if mesh_list
        .selected_descriptions
        .last()
        .map(|&last_index| !mesh_list.descriptions[last_index].commands.is_empty())
        .unwrap_or(true)
    {
        mesh_list
            .selected_descriptions
            .push(mesh_list.descriptions.len());

        // Ensure we generate a unique name
        let mut unique_name = "empty".to_string();
        let mut add_number = 1;
        while mesh_list
            .descriptions
            .iter()
            .position(|desc| desc.name == unique_name)
            .is_some()
        {
            unique_name = format!("empty {}", add_number);
            add_number += 1;
        }

        mesh_list.descriptions.push(MeshDescription {
            name: unique_name,
            is_renaming: false,
            commands: Vec::new(),
            selected_index: 0,
            current_mesh: Mesh::new(4),
            partial_mesh: Mesh::new(4),
        });
    }
}

enum MeshDescriptionInteraction {
    None,
    Hidden,
    Deleted,
    Changed,
}

fn draw_mesh_description(
    descriptions: &mut [MeshDescription],
    description_index: &mut usize,
    is_selected: &mut bool,
) -> MeshDescriptionInteraction {
    unsafe {
        igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));

        if *is_selected {
            igPushStyleColor(ImGuiCol::ChildBg, ImVec4::new(0.1, 0.1, 0.15, 1.));
        }
    }
    let show_child = unsafe {
        igBeginChild(
            cstr!("mesh desc"),
            ImVec2::new(MESH_WIDTH, 0.),
            true,
            ImGuiWindowFlags::empty(),
        )
    };
    unsafe {
        igPopStyleVar(1);

        if *is_selected {
            igPopStyleColor(1);
        }
    }

    let mut interaction = MeshDescriptionInteraction::None;

    if show_child {
        if unsafe {
            igIsMouseClicked(0, false) && igIsWindowHovered(ImGuiHoveredFlags::ChildWindows)
        } {
            *is_selected = true;
        }

        unsafe { igPushItemWidth(MESH_WIDTH) };

        if descriptions[*description_index].is_renaming {
            let mut data_str = String::new();
            mem::swap(&mut data_str, &mut descriptions[*description_index].name);
            let mut data_bytes = data_str.into_bytes();
            data_bytes.push(0);
            data_bytes.resize(data_bytes.len() + 10, 0); // reserve space for 10 more chars

            unsafe {
                igSetKeyboardFocusHere(0);
            }

            if unsafe {
                igInputText(
                    cstr!(""),
                    &mut data_bytes[0] as *mut u8 as *mut i8,
                    data_bytes.capacity(),
                    ImGuiInputTextFlags::EnterReturnsTrue | ImGuiInputTextFlags::AutoSelectAll,
                    None,
                    ptr::null_mut(),
                )
            } {
                descriptions[*description_index].is_renaming = false;
            }

            // Remove the first null char and everything after it
            for char_index in 0..data_bytes.len() {
                if data_bytes[char_index] == 0 {
                    data_bytes.truncate(char_index);
                    break;
                }
            }

            descriptions[*description_index].name = String::from_utf8(data_bytes).unwrap();
        } else {
            let mut current_name =
                CString::new(&descriptions[*description_index].name as &str).unwrap();
            if unsafe {
                igBeginCombo(
                    cstr!("##name"),
                    current_name.as_ptr(),
                    ImGuiComboFlags::empty(),
                )
            } {
                for (dropdown_index, description) in descriptions.iter_mut().enumerate() {
                    let label = CString::new(&description.name as &str).unwrap();
                    unsafe { igPushIDInt(dropdown_index as i32) };
                    if unsafe { igMenuItemBool(label.as_ptr(), ptr::null_mut(), false, true) } {
                        *description_index = dropdown_index;
                    }
                    unsafe { igPopID() };
                }

                unsafe { igSeparator() };

                if unsafe { igMenuItemBool(cstr!("Rename"), ptr::null_mut(), false, true) } {
                    descriptions[*description_index].is_renaming = true;
                }

                if unsafe { igMenuItemBool(cstr!("Hide"), ptr::null_mut(), false, true) } {
                    interaction = MeshDescriptionInteraction::Hidden;
                }

                // todo: disable delete if other lists depend on this one
                if unsafe { igMenuItemBool(cstr!("Delete"), ptr::null_mut(), false, true) } {
                    interaction = MeshDescriptionInteraction::Deleted;
                }

                unsafe { igEndCombo() };
            }
        }
        unsafe { igPopItemWidth() };

        // Display the list of commands
        let mut remove_index = None;
        for (command_index, command) in descriptions[*description_index].commands.iter().enumerate()
        {
            unsafe { igPushIDInt(command_index as i32) };

            let label = CString::new(command.ty.name()).unwrap();
            let is_selected =
                *is_selected && command_index == descriptions[*description_index].selected_index;
            if unsafe {
                igSelectable(
                    label.as_ptr(),
                    is_selected,
                    ImGuiSelectableFlags::empty(),
                    ImVec2::new(MESH_WIDTH - 26., 0.),
                )
            } {
                descriptions[*description_index].selected_index = command_index;
                interaction = MeshDescriptionInteraction::Changed;
            }
            unsafe {
                igSameLine(0., 4.);
                igSetCursorPosY(igGetCursorPosY() - 2.);
            }
            if unsafe { igButton(cstr!("X"), ImVec2::new(20., 17.)) } {
                remove_index = Some(command_index);
            }

            unsafe { igPopID() };
        }

        if let Some(remove_index) = remove_index {
            descriptions[*description_index]
                .commands
                .remove(remove_index);
        }

        unsafe {
            igSelectable(
                cstr!("+"),
                false,
                ImGuiSelectableFlags::empty(),
                ImVec2::new(0., 0.),
            )
        };
        if unsafe { igBeginPopupContextItem(cstr!("Select Command"), 0) } {
            for &command_type in COMMAND_TYPES {
                let label = CString::new(command_type.name()).unwrap();
                if unsafe { igMenuItemBool(label.as_ptr(), ptr::null_mut(), false, true) } {
                    let description = &mut descriptions[*description_index];

                    description.selected_index += 1;
                    if description.selected_index > description.commands.len() {
                        description.selected_index = description.commands.len();
                    }

                    description.commands.insert(
                        description.selected_index,
                        MeshCommand {
                            ty: command_type,
                            data: command_type.instantiate(),
                        },
                    );

                    interaction = MeshDescriptionInteraction::Changed;
                }
            }

            unsafe { igEndPopup() };
        }
    }
    unsafe {
        igEndChild();
    }

    interaction
}

fn draw_mesh_prop_panel(mesh_list: &mut MeshList) {
    if mesh_list.selected_index >= mesh_list.selected_descriptions.len() {
        return;
    }

    let selected_description =
        &mut mesh_list.descriptions[mesh_list.selected_descriptions[mesh_list.selected_index]];
    if selected_description.selected_index >= selected_description.commands.len() {
        return;
    }

    let selected_command = &mut selected_description.commands[selected_description.selected_index];

    let description_name_c_str = CString::new(&selected_description.name as &str).unwrap();
    let command_name_c_str = CString::new(selected_command.ty.name()).unwrap();

    unsafe {
        igSetCursorPosY(igGetCursorPosY() + 5.);
        igIndent(5.);
        igText(
            cstr!("%s -> %s"),
            description_name_c_str.as_ptr(),
            command_name_c_str.as_ptr(),
        );
    };

    let schema = selected_command.ty.schema();
    let data_bytes = unsafe { selected_command.data.as_bytes_mut() };
    for (property_index, property) in schema.properties.iter().enumerate() {
        unsafe { igPushIDInt(property_index as i32) };
        let cursor_pos = unsafe { igGetCursorPos_nonUDT2() };
        let prop_name_c_str = CString::new(property.name).unwrap();
        unsafe {
            igText(cstr!("%s"), prop_name_c_str.as_ptr());
            igSetCursorPosX(cursor_pos.x);
            igPushItemWidth(MESH_WIDTH - 10.);
        }
        match property.val_type {
            EditorCommandPropertyType::Signed => unsafe {
                igDragInt(
                    cstr!(""),
                    property.signed_mut(data_bytes),
                    1.,
                    0,
                    0,
                    cstr!("%d"),
                );
            },
            EditorCommandPropertyType::Unsigned => {
                let unsigned_val = unsafe { property.unsigned_mut(data_bytes) };
                let mut signed_val = *unsigned_val as i32;
                unsafe {
                    igDragInt(cstr!(""), &mut signed_val, 1., 0, 0, cstr!("%d"));
                }
                *unsigned_val = signed_val.max(0) as u32;
            }
            EditorCommandPropertyType::Float => unsafe {
                igDragFloat(
                    cstr!(""),
                    property.float_mut(data_bytes),
                    0.01,
                    0.,
                    0.,
                    cstr!("%.3f"),
                    2.,
                );
            },
            EditorCommandPropertyType::Vec2 => unsafe {
                igDragFloat2(
                    cstr!(""),
                    &mut property.vec2_mut(data_bytes).x,
                    0.01,
                    0.,
                    0.,
                    cstr!("%.3f"),
                    2.,
                );
            },
            EditorCommandPropertyType::Vec3 => unsafe {
                igDragFloat3(
                    cstr!(""),
                    &mut property.vec3_mut(data_bytes).x,
                    0.01,
                    0.,
                    0.,
                    cstr!("%.3f"),
                    2.,
                );
            },
            EditorCommandPropertyType::Vec4 => unsafe {
                igDragFloat4(
                    cstr!(""),
                    &mut property.vec4_mut(data_bytes).x,
                    0.01,
                    0.,
                    0.,
                    cstr!("%.3f"),
                    2.,
                );
            },
            EditorCommandPropertyType::RgbColor => unsafe {
                igColorEdit3(
                    cstr!(""),
                    &mut property.rgb_color_mut(data_bytes).0.x,
                    ImGuiColorEditFlags::RGB
                        | ImGuiColorEditFlags::Float
                        | ImGuiColorEditFlags::PickerHueWheel
                        | ImGuiColorEditFlags::HDR,
                );
            },
            EditorCommandPropertyType::RgbaColor => unsafe {
                igColorEdit4(
                    cstr!(""),
                    &mut property.rgba_color_mut(data_bytes).0.x,
                    ImGuiColorEditFlags::RGB
                        | ImGuiColorEditFlags::Float
                        | ImGuiColorEditFlags::AlphaBar
                        | ImGuiColorEditFlags::PickerHueWheel
                        | ImGuiColorEditFlags::AlphaPreview
                        | ImGuiColorEditFlags::HDR,
                );
            },
            EditorCommandPropertyType::Rotation => {
                // todo: find a nicer way to do this
                let quat_val = unsafe { property.rotation_mut(data_bytes) };
                let euler = quat_val.as_euler();
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
                    *quat_val = Quaternion::euler(
                        euler_degrees.0.to_radians(),
                        euler_degrees.1.to_radians(),
                        euler_degrees.2.to_radians(),
                    );
                }
            }
            EditorCommandPropertyType::Reference => {
                // todo
            }
        }
        unsafe {
            igPopItemWidth();
            igSetCursorPos(ImVec2::new(cursor_pos.x + MESH_WIDTH, cursor_pos.y));
            igPopID();
        }
    }
}
