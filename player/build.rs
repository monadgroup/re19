extern crate cc;
extern crate windres;

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let support_dir = std::path::Path::new(&dir).join("support");

    cc::Build::new()
        .file("support/win32_crt_math.cpp")
        .compile("win32_crt_math");

    windres::Build::new()
        .compile("resources/dialog.rc")
        .unwrap();

    println!("cargo:rustc-link-arg=/NODEFAULTLIB");
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/SAFESEH:NO");
    println!("cargo:rustc-link-arg=/DYNAMICBASE:NO");
    println!("cargo:rustc-link-arg=/ENTRY:WinMainCRTStartup");
    println!("cargo:rustc-link-arg=/LTCG");
    println!("cargo:rustc-link-search={}", support_dir.display());
    println!("cargo:rustc-link-lib=msvcrt");
    println!("cargo:rustc-link-lib=d3dx11");
    println!("cargo:rustc-link-lib=shlwapi");
}
