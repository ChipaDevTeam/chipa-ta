#[cfg(feature = "js")]
extern crate napi_build;

// #[cfg(feature = "py")]
// use std::process::Command;

fn main() {
    #[cfg(feature = "js")]
    napi_build::setup();
    // #[cfg(feature = "py")]
    // py_build();
}

// #[cfg(feature = "py")]
// fn py_build() {
//     let status = Command::new("cargo")
//         .arg("run")
//         .arg("--bin")
//         .arg("stubgen")  // Specify the binary you want to build
//         .arg("--features")
//         .arg("py") // Pass the feature you want to enable
//         .status()
//         .expect("Failed to execute command");

//     if !status.success() {
//         panic!("Cargo build failed!");
//     }

// }