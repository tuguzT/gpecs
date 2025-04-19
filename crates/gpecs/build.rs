use std::{path, process::Command};

use const_format::formatcp;

fn main() {
    let output = Command::new("cargo")
        .arg("gpu")
        .arg("build")
        .arg("--auto-install-rust-toolchain")
        .arg("--force-spirv-cli-rebuild")
        .arg("--shader-crate")
        .arg("./../gpecs_shader_example")
        .arg("--output-dir")
        .arg("./../gpecs_shader_example")
        .output()
        .expect("failed to build shaders");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("failed to build shaders:\n{stderr}");
    }

    const SHADER_CRATE_NAME: &str = "gpecs_shader_example";
    const SHADER_REL_PATH: &str = formatcp!("./../{SHADER_CRATE_NAME}/{SHADER_CRATE_NAME}.spv");

    let shader_abs_path = path::absolute(SHADER_REL_PATH).expect("failed to get absolute path");
    let shader_abs_path = shader_abs_path.display();

    println!("cargo::rustc-env={SHADER_CRATE_NAME}.spv={shader_abs_path}");
    println!("cargo::rerun-if-changed=./crates/{SHADER_CRATE_NAME}");
}
