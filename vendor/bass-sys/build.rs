use std::env;

fn main() {
    println!(r#"cargo:rustc-link-search=native={}"#, env::current_dir().unwrap().to_str().unwrap());
    println!("cargo:rustc-link-lib=bass");
}
