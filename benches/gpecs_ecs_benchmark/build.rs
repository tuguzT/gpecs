use std::{path, process::Command};

use const_format::formatcp;

const SHADER_CRATE_NAME: &str = "gpecs_ecs_benchmark_shader";
const SHADER_CRATE_PATH: &str = formatcp!("./../{SHADER_CRATE_NAME}");

fn main() {
    let output = Command::new("cargo")
        .arg("gpu")
        .arg("build")
        .arg("--auto-install-rust-toolchain")
        .arg("--force-spirv-cli-rebuild")
        .arg("--shader-crate")
        .arg(SHADER_CRATE_PATH)
        .arg("--output-dir")
        .arg(SHADER_CRATE_PATH)
        .output()
        .expect("failed to build shaders");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("failed to build shaders:\n{stderr}");
    }

    const SHADER_FILE_PATH: &str = formatcp!("{SHADER_CRATE_PATH}/{SHADER_CRATE_NAME}.spv");
    let shader_file_path = path::absolute(SHADER_FILE_PATH).expect("failed to get absolute path");
    let shader_file_path = shader_file_path.display();

    println!("cargo::rustc-env={SHADER_CRATE_NAME}.spv={shader_file_path}");
    println!("cargo::rerun-if-changed={SHADER_CRATE_PATH}");
}
