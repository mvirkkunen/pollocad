fn main() {
    cpp_build::Config::new()
        .flag_if_supported("-std=c++14")
        .include("src/")
        .include("/usr/include/opencascade")
        .build("src/lib.rs");

    println!("cargo:rerun-if-changed=src/");
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
