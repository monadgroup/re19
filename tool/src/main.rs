#![windows_subsystem = "windows"]

use chrono::Utc;
use imgui_sys::{
    igBegin, igDockSpace, igEnd, igGetIDStr, igGetMainViewport, igPopStyleVar, igPushStyleVarFloat,
    igPushStyleVarVec2, igSetNextWindowPos, igSetNextWindowSize, igSetNextWindowViewport,
    ImGuiCond, ImGuiDockNodeFlags, ImGuiStyleVar, ImGuiWindowFlags, ImVec2,
};
use ron::de::Deserializer;
use ron::ser::{PrettyConfig, Serializer};
use std::env::current_exe;
use std::time::Instant;
use std::{mem, ptr};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    CreateWindowExA, RegisterClassA, CS_OWNDC, WNDCLASSA, WS_EX_OVERLAPPEDWINDOW,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

#[macro_use]
mod cstr;

mod audio;
mod editor_clip_map;
mod editor_state;
mod exporter;
mod imgui;
mod imgui_window;
mod panels;
//mod recycle_bin;
//mod mesh_list;
mod serialize;
mod timeline_interactions;

use crate::editor_state::EditorState;
use crate::imgui_window::ImGuiWindow;
use crate::serialize::{deserialize_timeline, serialize_timeline};
use engine::animation::timeline::{Timeline, Track};
use engine::creation_context::CreationContext;
use engine::frame_context::CommonData;
use engine::gbuffer::GBuffer;
use engine::renderer::RendererCollection;
use engine::resources::perf_table::PerfTable;
use engine::resources::shader_manager::ShaderManager;
use engine::viewport::Viewport;
use path_abs::PathDir;
use std::fs;
use std::path::Path;

fn main() {
    engine::math::random::seed_rand(0x1337b012);

    let class_name = cstr!("You lost the game");
    let inst = unsafe { GetModuleHandleA(ptr::null()) };

    // create and register the window class
    let mut window_class = unsafe { mem::zeroed::<WNDCLASSA>() };
    window_class.style = CS_OWNDC;
    window_class.hInstance = inst;
    window_class.lpszClassName = class_name;
    window_class.lpfnWndProc = Some(imgui_window::ImGuiWindow::wnd_proc);
    unsafe {
        RegisterClassA(&window_class);
    }

    let dw_style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
    let dw_ex_style = WS_EX_OVERLAPPEDWINDOW;

    let hwnd = unsafe {
        CreateWindowExA(
            dw_ex_style,
            class_name,
            class_name,
            dw_style,
            0,
            0,
            1280,
            920,
            ptr::null_mut(),
            ptr::null_mut(),
            inst,
            ptr::null_mut(),
        )
    };

    let mut project_path = current_exe().unwrap(); // "re19/target/debug/tool.exe"
    project_path.pop(); // "re19/target/debug/"
    project_path.pop(); // "re19/target/"
    project_path.pop(); // "re19/"
    project_path.push("project"); // "re19/project/
    let shader_path = project_path.join("shaders");
    let saves_path = project_path.join("saves");

    let mut window = ImGuiWindow::new(hwnd);

    // note: should match viewport in player
    let aspect = 2560. / 1080.;
    let project_viewport = Viewport {
        width: 1920,
        height: (1920. / aspect) as u32,
    };

    let mut shader_manager = ShaderManager::new(PathDir::new(shader_path).unwrap());
    let mut creation_context = CreationContext {
        device: window.resources.device(),
        devcon: window.resources.devcon(),
        shader_manager: &mut shader_manager,
        viewport: project_viewport,
    };
    let mut renderers = RendererCollection::new(&mut creation_context);
    let mut common = CommonData::new(&mut creation_context);
    let mut gbuffer = GBuffer::new(creation_context.device, project_viewport);
    let mut perf_table = PerfTable::new(creation_context.device, creation_context.devcon);

    // Try to load the timeline from a file
    let head_save_path = saves_path.join("000000-000000-head.save");
    let mut timeline = match fs::read_to_string(&head_save_path) {
        Ok(file_content) => {
            // save a backup in case deserialization causes problems
            save_backup(&saves_path, &head_save_path, &file_content);

            let mut deserializer = Deserializer::from_str(&file_content).unwrap();
            deserialize_timeline(&mut deserializer, &mut creation_context).unwrap()
        }
        Err(_) => Timeline {
            tracks: vec![Track::default()],
        },
    };

    /*let mut mesh_list = mesh_list::MeshList {
        descriptions: Vec::new(),
        selected_descriptions: Vec::new(),
        selected_index: 0,
    };*/

    let mut audio_path = current_exe().unwrap(); // "re19/target/debug/tool.exe"
    audio_path.pop(); // "re19/target/debug/"
    audio_path.pop(); // "re19/target/"
    audio_path.pop(); // "re19/"
    audio_path.push("audio.ogg");
    let mut audio_player = audio::BassPlayer::new(audio_path.to_str().unwrap()).unwrap();

    // note: should match framerate in player
    let mut editor_state = EditorState::new(60., 112., 4, &mut audio_player);

    // Set the editor's next ID to the next highest one
    editor_state.next_clip_id = timeline
        .tracks
        .iter()
        .flat_map(|track| track.clips.iter())
        .fold(0, |next_id, clip| next_id.max(clip.id + 1));

    let mut last_save_time = Instant::now();
    let mut last_frame_time = Instant::now();

    loop {
        let mut this_frame_time = Instant::now();
        let frame_duration = this_frame_time.duration_since(last_frame_time);
        last_frame_time = this_frame_time;

        perf_table.begin_frame();
        editor_state.update();
        window.start_frame();
        unsafe {
            // create a fullscreen dockspace
            let viewport = igGetMainViewport();
            igSetNextWindowPos((*viewport).pos, ImGuiCond::empty(), ImVec2::new(0., 0.));
            igSetNextWindowSize((*viewport).size, ImGuiCond::empty());
            igSetNextWindowViewport((*viewport).id);
            igPushStyleVarFloat(ImGuiStyleVar::WindowRounding, 0.);
            igPushStyleVarFloat(ImGuiStyleVar::WindowBorderSize, 0.);
            igPushStyleVarVec2(ImGuiStyleVar::WindowPadding, ImVec2::new(0., 0.));

            let window_flags = ImGuiWindowFlags::MenuBar
                | ImGuiWindowFlags::NoDocking
                | ImGuiWindowFlags::NoTitleBar
                | ImGuiWindowFlags::NoCollapse
                | ImGuiWindowFlags::NoResize
                | ImGuiWindowFlags::NoMove
                | ImGuiWindowFlags::NoBringToFrontOnFocus
                | ImGuiWindowFlags::NoNavFocus;

            igBegin(cstr!("Dockspace"), ptr::null_mut(), window_flags);
            igPopStyleVar(3);
            igDockSpace(
                igGetIDStr(cstr!("dock")),
                ImVec2::new(0., 0.),
                ImGuiDockNodeFlags::empty(),
                ptr::null(),
            );
            igEnd();
        }

        shader_manager.update(window.resources.device());

        // Processing order: (this is important!)
        //  - Draw timeline editor (this might modify the timeline, so we do it first so
        //    everything else is consistent)
        //  - Draw motion editor
        //  - Build/update clip-property map (a mapping from each clip ID to the clip object,
        //    and allocated space for finalised property values)
        //  - Coallesce all values (combines animation clips with their target properties,
        //    and builds any other info needed by the property editor)
        //  - Render the current clips

        let cpu_ui_query = perf_table.start_cpu_str("ui (cpu)");
        panels::draw_profiler(&mut perf_table);
        panels::draw_timeline(
            &mut timeline,
            &mut editor_state,
            &mut CreationContext {
                device: window.resources.device(),
                devcon: window.resources.devcon(),
                shader_manager: &mut shader_manager,
                viewport: project_viewport,
            },
        );
        let clip_map_query = perf_table.start_cpu_str("build clip map");
        let mut clip_map =
            editor_clip_map::EditorClipMap::from_timeline(&timeline, editor_state.current_frame());
        perf_table.end(clip_map_query);
        panels::draw_motion_editor(&mut timeline, &mut editor_state, &clip_map);
        let animation_query = perf_table.start_cpu_str("animation");
        engine::animation::coallesce::coallesce_animations(&timeline, &mut clip_map);
        perf_table.end(animation_query);
        panels::draw_property_editor(&mut timeline, &clip_map, &mut editor_state);
        perf_table.end(cpu_ui_query);

        let cpu_frame_query = perf_table.start_cpu_str("frame (cpu)");
        let frame_query = perf_table.start_gpu_str("frame (gpu)");
        panels::draw_preview(
            window.resources.devcon(),
            frame_duration.as_secs_f32(),
            project_viewport,
            &shader_manager,
            &mut timeline,
            &clip_map,
            &mut renderers,
            &mut gbuffer,
            &mut common,
            &mut perf_table,
            &mut editor_state,
        );
        perf_table.end(frame_query);
        perf_table.end(cpu_frame_query);

        let gpu_ui_query = perf_table.start_gpu_str("ui (gpu)");
        window.end_frame();
        perf_table.end(gpu_ui_query);
        perf_table.end_frame();

        editor_state.post_update();

        let is_open = window.poll_events();
        if !is_open {
            break;
        }

        // If five minutes have passed since saving, make a new save
        // If there are more than 100 saves are in the folder (corresponding to roughly 5.3 hours),
        // delete the oldest one
        if last_save_time.elapsed().as_secs() >= 5 * 60 {
            last_save_time = Instant::now();
            save_backup(&saves_path, &head_save_path, &timeline_to_string(&timeline));
        }
    }

    // Save the timeline to disk
    fs::write(&head_save_path, timeline_to_string(&timeline)).unwrap();

    // Export the data and save that to disk
    let mut export = Vec::new();
    exporter::export_shaders(&shader_manager, &mut export);
    exporter::export_timeline(&timeline, &mut export);
    fs::write(project_path.join("data.blob"), &export).unwrap();
}

fn save_backup(saves_path: &Path, head_save_path: &Path, timeline_str: &str) {
    let current_date_time = Utc::now();
    let save_file_name = saves_path.join(
        current_date_time
            .format("%y%m%d-%H%M%S-backup.save")
            .to_string(),
    );
    fs::write(&save_file_name, timeline_str).unwrap();
    println!(
        "Saved backup project file as {}",
        save_file_name.to_str().unwrap()
    );

    // If there are >100 entries, delete the oldest one (as long as it's not our head save)
    let mut non_head_entries: Vec<_> = fs::read_dir(&saves_path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.path() == head_save_path {
                None
            } else {
                Some(entry)
            }
        })
        .collect();

    // If there are >100 entries, we'll need to delete the oldest one
    if non_head_entries.len() > 100 {
        // Sort the entries by name, so the first one is the oldest
        non_head_entries.sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));

        let first_file_name = non_head_entries.first().unwrap().path();
        fs::remove_file(&first_file_name).unwrap();
        println!(
            "Deleted old backup file {}",
            first_file_name.to_str().unwrap()
        );
    }
}

fn timeline_to_string(timeline: &Timeline) -> String {
    let mut serializer = Serializer::new(
        Some(PrettyConfig {
            depth_limit: 20,
            new_line: "\n".to_string(),
            indentor: "  ".to_string(),
            separate_tuple_members: false,
            enumerate_arrays: false,
        }),
        false,
    );
    serialize_timeline(&timeline, &mut serializer).unwrap();
    serializer.into_output_string()
}
