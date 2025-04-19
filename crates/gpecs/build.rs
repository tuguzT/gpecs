use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .arg("gpu")
        .arg("build")
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
    print!(
        r#"
        cargo::rustc-env={SHADER_CRATE_NAME}.spv=./crates/{SHADER_CRATE_NAME}/{SHADER_CRATE_NAME}.spv
        cargo::rerun-if-changed=./crates/{SHADER_CRATE_NAME}
        "#
    );
}
