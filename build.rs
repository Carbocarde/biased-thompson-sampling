use std::path::PathBuf;
use std::process::Command;

fn main() {
    let status = Command::new("mkdir")
        .args(["-p", "./cc_shared_libs"])
        .status()
        .expect("Failed to make library output folder");

    if !status.success() {
        panic!("Failed to make library output folder");
    }

    // Compile the C++ code into a shared library
    let status = Command::new("g++")
        .args([
            "-shared",
            "-fPIC",
            "-I./ffi/boost_1_82_0",
            "./ffi/boost_wrapper.cc",
            "-L./ffi/boost_1_82_0/libs",
            "-o",
            "./cc_shared_libs/libboost_wrapper.so",
        ])
        .status()
        .expect("Failed to compile boost_wrapper.cpp");

    if !status.success() {
        panic!("Failed to compile boost_wrapper.cpp");
    }

    // Specify the path to the directory containing the shared library
    let lib_path = PathBuf::from("./cc_shared_libs");
    println!("cargo:rustc-link-search={}", lib_path.display());

    // Link against the compiled shared library
    println!("cargo:rustc-link-lib=dylib=boost_wrapper");
}
