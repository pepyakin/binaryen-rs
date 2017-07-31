extern crate bindgen;
extern crate cmake;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn gen_bindings() {
    let bindings = bindgen::Builder::default()
        .header("binaryen/src/binaryen-c.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    if !Path::new("binaryen/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
                                   .status();
    }

    gen_bindings();

    let dst = cmake::Config::new("binaryen")
        .define("BUILD_STATIC_LIB", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-lib=static=binaryen");
    println!("cargo:rustc-link-lib=static=asmjs");
    println!("cargo:rustc-link-lib=static=ast");
    println!("cargo:rustc-link-lib=static=cfg");
    println!("cargo:rustc-link-lib=static=passes");
    println!("cargo:rustc-link-lib=static=support");
    println!("cargo:rustc-link-lib=static=wasm");
    println!("cargo:rustc-link-lib=static=emscripten-optimizer");
    println!("cargo:rustc-flags=-l dylib=c++");
}
