use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_language(cbindgen::Language::C)
        .with_crate(crate_dir)
        .generate()
        .expect("generating bindings")
        .write_to_file("lib/runtime.h");

    // move the libruntime to the lib directory with os specific extension
    let lib_name = get_dylib_name();
    let source = PathBuf::from("target/release").join(&lib_name);
    let target = PathBuf::from("lib").join(&lib_name);
    println!("cargo:warning=Copying library from {:?} to {:?}", source, target);
    fs::copy(&source, &target).expect(&format!("Failed to copy library from {:?} to {:?}", source, target));
}

fn get_dylib_name() -> String {
    #[cfg(target_os = "linux")]
    return "libruntime.so".to_string();

    #[cfg(target_os = "macos")]
    return "libruntime.dylib".to_string();

    #[cfg(target_os = "windows")]
    return "runtime.dll".to_string();
}
