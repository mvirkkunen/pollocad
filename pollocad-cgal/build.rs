use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++14") // add this
        .include("cpp/")
        .file("cpp/wrapper.cpp")
        .compile("pollocad_cgal");

    let bindings = bindgen::Builder::default()
        .header("cpp/wrapper.h")
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=cpp/wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/wrapper.h");
    println!("cargo:rustc-link-lib=pollocad_cgal");
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rustc-link-lib=mpfr");
}
