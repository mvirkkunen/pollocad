use std::path::PathBuf;
use std::env;

fn main() {
    let occt_install_path = env::var("OCCT_INSTALL_PATH");
    
    let mut build = cpp_build::Config::new();

    build
        .flag_if_supported("-std=c++14")
        .include("src/");

    println!("path: {:?}", occt_install_path);

    if let Ok(path) = occt_install_path {
        build.include(&format!("{}/include/opencascade", path));
        println!("cargo:rustc-link-search=native={}/lib", path);
    }
    
    build.build("src/lib.rs");

    let bindings = bindgen::Builder::default()
        .header("src/constants.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustified_enum("BooleanOp")
        .constified_enum_module("MouseFlags")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings_constants.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rustc-link-lib=dylib=TKBO");
    println!("cargo:rustc-link-lib=dylib=TKBRep");
    println!("cargo:rustc-link-lib=dylib=TKernel");
    println!("cargo:rustc-link-lib=dylib=TKMath");
    println!("cargo:rustc-link-lib=dylib=TKOpenGl");
    println!("cargo:rustc-link-lib=dylib=TKPrim");
    println!("cargo:rustc-link-lib=dylib=TKService");
    println!("cargo:rustc-link-lib=dylib=TKTopAlgo");
    println!("cargo:rustc-link-lib=dylib=TKV3d");
    println!("cargo:rustc-link-lib=dylib=GL");
}

// for x in /usr/lib/*TK*.so; do echo $x; nm --dynamic $x|grep y; done|grep -B1 "T "
