use std::error::Error;

use cargo_gpu_install::{install::Install, spirv_builder::SpirvMetadata};
use const_format::formatcp;

const SHADER_CRATE_NAME: &str = "gpecs_simple_shader";
const SHADER_CRATE_PATH: &str = formatcp!("./../{SHADER_CRATE_NAME}");

fn main() -> Result<(), Box<dyn Error>> {
    let backend_args = Install::from_shader_crate(SHADER_CRATE_PATH.into());
    let backend = backend_args.run()?;

    let builder = backend
        .to_spirv_builder(SHADER_CRATE_PATH, "spirv-unknown-vulkan1.2")
        .shader_crate_default_features(false)
        .shader_crate_features(["nightly".into()])
        // .release(std::env::var("CARGO_CFG_DEBUG_ASSERTIONS").is_err())
        .spirv_metadata(SpirvMetadata::Full);
    let compile_result = builder.build()?;

    let shader_file_path = compile_result.module.unwrap_single().display();
    println!("cargo::rustc-env={SHADER_CRATE_NAME}.spv={shader_file_path}");
    println!("cargo::rerun-if-changed={SHADER_CRATE_PATH}");

    Ok(())
}
