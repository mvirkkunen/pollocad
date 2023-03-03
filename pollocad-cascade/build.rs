use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++14") // add this
        .include("cpp/")
        .include("/usr/include/opencascade/")
        .files(&[
            "cpp/shape.cpp",
            "cpp/preview.cpp",
            "cpp/util.cpp",
        ])
        .compile("pollocad_cascade");

    let bindings = bindgen::Builder::default()
        .header("cpp/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=cpp/shape.cpp");
    println!("cargo:rerun-if-changed=cpp/preview.cpp");
    println!("cargo:rerun-if-changed=cpp/util.cpp");
    println!("cargo:rerun-if-changed=cpp/util.hpp");
    println!("cargo:rustc-link-lib=TKBO");
    println!("cargo:rustc-link-lib=TKBRep");
    println!("cargo:rustc-link-lib=TKernel");
    println!("cargo:rustc-link-lib=TKMath");
    println!("cargo:rustc-link-lib=TKOpenGl");
    println!("cargo:rustc-link-lib=TKPrim");
    println!("cargo:rustc-link-lib=TKService");
    println!("cargo:rustc-link-lib=TKTopAlgo");
    println!("cargo:rustc-link-lib=TKV3d");
}

// for x in /usr/lib/*TK*.so; do echo $x; nm --dynamic $x|grep y; done|grep -B1 "T "
