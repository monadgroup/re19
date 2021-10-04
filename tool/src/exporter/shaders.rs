use super::binary_writer::write;
use engine::resources::shader_manager::ShaderManager;
use lazy_static::lazy_static;
use path_abs::PathDir;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::env::current_exe;
use std::fs;
use std::os::windows::process::CommandExt;
use std::process::Command;
use winapi::um::winbase::CREATE_NO_WINDOW;

const MINIFY_SHADERS: bool = true;

fn load_shader_str<'cache>(
    path: &str,
    is_entry_point: bool,
    str_cache: &'cache mut HashMap<String, String>,
) -> &'cache str {
    str_cache.entry(path.to_string()).or_insert_with(|| {
        if MINIFY_SHADERS && !path.ends_with("discard_sky.hlsl") {
            let mut minifier_exe_path = current_exe().unwrap(); // "re19/target/debug/tool.exe"
            minifier_exe_path.pop(); // "re19/target/debug/"
            minifier_exe_path.pop(); // "re19/target/"
            minifier_exe_path.pop(); // "re19/"
            minifier_exe_path.push("shader_minifier.exe"); // "re19/shader_minifier.exe

            let trimmed_path = path.trim_start_matches("\\\\?\\");
            println!("Minifying {}...", trimmed_path);

            let mut args = vec![
                trimmed_path,
                "-o",
                "export_shader_output.hlsl",
                "--hlsl",
                "--format",
                "none",
            ];
            if is_entry_point {
                args.extend_from_slice(&["--no-renaming-list", "main,MAX_ITERATIONS"]);
            } else {
                args.extend_from_slice(&["--preserve-all-globals"]);
            }

            let status = Command::new(minifier_exe_path)
                .args(&args)
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .unwrap();
            if status.success() {
                fs::read_to_string("export_shader_output.hlsl").unwrap()
            } else {
                eprintln!("Failed to minify shader, using unminified version");
                fs::read_to_string(path).unwrap()
            }
        } else {
            fs::read_to_string(path).unwrap()
        }
    })
}

fn process_shader(
    source: &str,
    dir: &PathDir,
    processed_shader_strings: &mut Vec<String>,
    shader_map: &mut HashMap<String, usize>,
    str_cache: &mut HashMap<String, String>,
) -> usize {
    lazy_static! {
        static ref INCLUDE_RE: Regex = Regex::new(r#"#include\s*"(.*)""#).unwrap();
    }

    if let Some(&existing_index) = shader_map.get(source) {
        return existing_index;
    }

    // Reserve a slot in the processed shader strings map, we'll fill it later
    // (this just means we don't get into an infinite loop with recursive dependencies)
    let string_slot = processed_shader_strings.len();
    shader_map.insert(source.to_string(), string_slot);
    processed_shader_strings.push("".to_string());

    // Walk to any #include "..." targets, and replace the inner value with the string index
    let processed_str = INCLUDE_RE
        .replace_all(source, |caps: &Captures| {
            let fixed_path = caps[1].replace("/", "\\");
            let resolved_file = dir
                .join(&fixed_path)
                .absolute()
                .unwrap()
                .into_file()
                .unwrap();
            let dependency_src =
                load_shader_str(resolved_file.to_str().unwrap(), false, str_cache).to_string();
            let shader_index = process_shader(
                &dependency_src,
                &resolved_file.parent_dir().unwrap(),
                processed_shader_strings,
                shader_map,
                str_cache,
            );
            format!(r#"#include "{}""#, shader_index)
        })
        .into_owned();

    processed_shader_strings[string_slot] = processed_str;
    string_slot
}

pub fn export_shaders(shader_manager: &ShaderManager, buffer: &mut Vec<u8>) {
    let path_journal = shader_manager.path_journal();

    let mut processed_shader_strings = Vec::new();
    let mut shader_map = HashMap::new();
    let mut entry_point_map = HashMap::new();
    let mut str_cache = HashMap::new();

    let mut entry_point_count = 0u8;
    let mut creation_indices_stream = Vec::new();
    let mut entry_point_types_stream = Vec::new();
    let mut entry_point_indices_stream = Vec::new();
    let mut string_length_stream = Vec::new();
    let mut string_stream = Vec::new();

    for (entry_point_type, entry_point_path) in path_journal {
        let creation_index = *entry_point_map
            .entry((*entry_point_type, entry_point_path))
            .or_insert_with(|| {
                let entry_point_src =
                    load_shader_str(entry_point_path.to_str().unwrap(), false, &mut str_cache)
                        .to_string();
                let entry_point_index = process_shader(
                    &entry_point_src,
                    &entry_point_path.parent_dir().unwrap(),
                    &mut processed_shader_strings,
                    &mut shader_map,
                    &mut str_cache,
                );

                write(&mut entry_point_types_stream, *entry_point_type as u8);
                write(&mut entry_point_indices_stream, entry_point_index as u8);

                let entry_point_id = entry_point_count;
                entry_point_count += 1;
                entry_point_id
            });

        write(&mut creation_indices_stream, creation_index);
    }

    for shader_string in &processed_shader_strings {
        write(&mut string_length_stream, shader_string.len() as u32);
        string_stream.extend_from_slice(shader_string.as_bytes());
    }

    write(buffer, creation_indices_stream.len() as u32);
    buffer.extend_from_slice(&creation_indices_stream);

    write(buffer, entry_point_types_stream.len() as u32);
    buffer.extend_from_slice(&entry_point_types_stream);

    write(buffer, entry_point_indices_stream.len() as u32);
    buffer.extend_from_slice(&entry_point_indices_stream);

    write(buffer, processed_shader_strings.len() as u8);
    buffer.extend_from_slice(&string_length_stream);
    buffer.extend_from_slice(&string_stream);
}
