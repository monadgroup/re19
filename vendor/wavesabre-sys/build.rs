use cmake::Config;

fn main() {
    let dst = Config::new("ws-lib")
        .build_target("WaveSabreC")
        .profile("Release")
        .build();
    let build_dir = dst.join("build");
    let ws_c_dir = build_dir.join("WaveSabreC").join("Release");
    let ws_player_lib_dir = build_dir.join("WaveSabrePlayerLib").join("Release");
    let ws_core_dir = build_dir.join("WaveSabreCore").join("Release");

    println!("cargo:rustc-link-search=native={}", ws_c_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        ws_player_lib_dir.display()
    );
    println!("cargo:rustc-link-search=native={}", ws_core_dir.display());

    println!("cargo:rustc-link-lib=static=WaveSabreC");
    println!("cargo:rustc-link-lib=static=WaveSabrePlayerLib");
    println!("cargo:rustc-link-lib=static=WaveSabreCore");
    println!("cargo:rustc-link-lib=winmm");
    println!("cargo:rustc-link-lib=dsound");
    println!("cargo:rustc-link-lib=uuid");
    println!("cargo:rustc-link-lib=msacm32");
}
