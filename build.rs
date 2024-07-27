use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=dylib=ArducamDepthCamera2c");

    let bindings = bindgen::Builder::default()
        .header("ArducamDepthCamera 0.1.3.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}