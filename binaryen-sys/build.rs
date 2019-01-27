extern crate bindgen;
extern crate cc;
extern crate cmake;
extern crate heck;
extern crate regex;

use heck::CamelCase;
use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

fn gen_bindings() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // See https://github.com/rust-lang-nursery/rust-bindgen/issues/947
        .trust_clang_mangling(false)
        .generate_comments(true)
        // https://github.com/rust-lang-nursery/rust-bindgen/issues/947#issuecomment-327100002
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[derive(Clone, PartialEq, Debug)]
struct Pass {
    id: String,
    name: String,
    description: String,
}

fn read_passes() -> Vec<Pass> {
    let re = Regex::new(r#"registerPass\("([^"]+)", "([^"]+)", [^)]+\);"#).unwrap();

    let mut passes: Vec<Pass> = vec![];

    let input = File::open("binaryen/src/passes/pass.cpp").expect("Couldn't open pass.cpp");
    for line in BufReader::new(input).lines() {
        let line = line.unwrap();
        let caps = re.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let name = caps.get(1).unwrap().as_str();
            let description = caps.get(2).unwrap().as_str();

            passes.push(Pass {
                id: name.to_camel_case(),
                name: name.to_string(),
                description: description.to_string(),
            });
        }
    }

    passes
}

fn gen_passes() {
    let passes: Vec<Pass> = read_passes();

    let ids: Vec<String> = passes
        .iter()
        .map(|pass| format!("/// {}\n{}", pass.description.to_string(), pass.id.to_string()))
        .collect();

    let fromstrs: Vec<String> = passes
        .iter()
        .map(|pass| format!(r#""{}" => Ok(OptimizationPass::{})"#, pass.name.to_string(), pass.id.to_string()))
        .collect();

    let descriptions: Vec<String> = passes
        .iter()
        .map(|pass| format!(r#"OptimizationPass::{} => "{}""#, pass.id.to_string(), pass.description.to_string()))
        .collect();

    let output = format!(r#"
        use std::str::FromStr;

        #[derive(Eq, PartialEq, Debug)]
        pub enum OptimizationPass {{
            {ids}
        }}

        impl FromStr for OptimizationPass {{
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {{
                match s {{
                    {fromstrs},
                    _ => Err(()),
                }}
            }}
        }}

        trait OptimizationPassDescription {{
            fn description(&self) -> &'static str;
        }}

        impl OptimizationPassDescription for OptimizationPass {{
            fn description(&self) -> &'static str {{
                match self {{
                    {descriptions}
                }}
            }}
        }}

        #[cfg(test)]
        mod tests {{
            use super::*;

            #[test]
            fn test_from_str() {{
                assert_eq!(OptimizationPass::{test_id}, OptimizationPass::from_str("{test_name}").expect("from_str expected to work"));
            }}

            #[test]
            fn test_description() {{
                assert_eq!(OptimizationPass::{test_id}.description(), "{test_description}");
            }}
        }}
    "#,
    ids = ids.join(",\n"),
    fromstrs = fromstrs.join(",\n"),
    descriptions = descriptions.join(",\n"),
    test_id = passes[0].id.to_string(),
    test_name = passes[0].name.to_string(),
    test_description = passes[0].description.to_string()
    );

    fs::write("src/passes.rs", output).expect("Unable to write passes.rs");
}

fn main() {
    if !Path::new("binaryen/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    gen_passes();
    gen_bindings();

    let target = env::var("TARGET").ok();
    if target.map_or(false, |target| target.contains("emscripten")) {
        let mut build_wasm_binaryen_args = vec![];
        if get_debug() {
            build_wasm_binaryen_args.push("-g");
        }

        let _ = Command::new("./build-binaryen-bc.sh")
            .args(&build_wasm_binaryen_args)
            .status()
            .unwrap();

        println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-link-lib=static=binaryen-c");
        return;
    }

    let dst = cmake::Config::new("binaryen")
        .define("BUILD_STATIC_LIB", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-lib=static=binaryen");
    println!("cargo:rustc-link-lib=static=asmjs");
    println!("cargo:rustc-link-lib=static=cfg");
    println!("cargo:rustc-link-lib=static=ir");
    println!("cargo:rustc-link-lib=static=passes");
    println!("cargo:rustc-link-lib=static=support");
    println!("cargo:rustc-link-lib=static=wasm");
    println!("cargo:rustc-link-lib=static=emscripten-optimizer");

    // We need to link against C++ std lib
    if let Some(cpp_stdlib) = get_cpp_stdlib() {
        println!("cargo:rustc-link-lib={}", cpp_stdlib);
    }

    let mut cfg = cc::Build::new();
    cfg.file("Shim.cpp")
        .include("binaryen/src")
        .cpp_link_stdlib(None)
        .warnings(false)
        .cpp(true)
        .flag("-std=c++11")
        .compile("binaryen_shim");
}

// See https://github.com/alexcrichton/gcc-rs/blob/88ac58e25/src/lib.rs#L1197
fn get_cpp_stdlib() -> Option<String> {
    env::var("TARGET").ok().and_then(|target| {
        if target.contains("msvc") {
            None
        } else if target.contains("darwin") {
            Some("c++".to_string())
        } else if target.contains("freebsd") {
            Some("c++".to_string())
        } else if target.contains("musl") {
            Some("static=stdc++".to_string())
        } else {
            Some("stdc++".to_string())
        }
    })
}

// See https://github.com/alexcrichton/gcc-rs/blob/10871a0e40/src/lib.rs#L1501
fn get_debug() -> bool {
    match env::var("DEBUG").ok() {
        Some(s) => s != "false",
        None => false,
    }
}
