use std::{env, path, process::Command};

use const_format::formatcp;

const SHADER_CRATE_NAME: &str = "gpecs_simple_shader";
const SHADER_CRATE_PATH: &str = formatcp!("./../{SHADER_CRATE_NAME}");

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = path::absolute(out_dir).expect("failed to get absolute path");
    let shader_crate_path = path::absolute(SHADER_CRATE_PATH).expect("failed to get absolute path");

    let output = Command::new("cargo")
        .arg("gpu")
        .arg("build")
        .arg("--auto-install-rust-toolchain")
        .arg("--shader-crate")
        .arg(&shader_crate_path)
        .arg("--output-dir")
        .arg(&out_dir)
        .output()
        .expect("failed to build shaders");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("failed to build shaders:\n{stderr}");
    }

    let shader_crate_path = shader_crate_path.display();

    let shader_file_path = out_dir.join(SHADER_CRATE_NAME).with_extension("spv");
    let shader_file_path = shader_file_path.display();

    println!("cargo::rustc-env={SHADER_CRATE_NAME}.spv={shader_file_path}");
    println!("cargo::rerun-if-changed={shader_crate_path}");
}
