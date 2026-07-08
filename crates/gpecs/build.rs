use std::{env, error::Error};

use cargo_gpu_install::{install::Install, spirv_builder::SpirvMetadata};
use cargo_metadata::MetadataCommand;

const SHADER_CRATE_NAME: &str = "gpecs_shaders";

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("manifest directory environment variable should be set by Cargo");
    let metadata = MetadataCommand::new()
        .current_dir(manifest_dir)
        .no_deps()
        .exec()?;
    let shader_crate = metadata
        .packages
        .iter()
        .find(|package| package.name == SHADER_CRATE_NAME)
        .expect("target shader crate was not found");
    let shader_crate_path = shader_crate
        .manifest_path
        .parent()
        .expect("manifest path of target shader crate shoyld have a parent");

    let backend_args = Install::from_shader_crate(shader_crate_path.into());
    let backend = backend_args.run()?;

    let mut builder = backend
        .to_spirv_builder(shader_crate_path, "spirv-unknown-vulkan1.2")
        .shader_crate_default_features(false)
        .shader_crate_features(["nightly".into()])
        .spirv_metadata(SpirvMetadata::Full);
    builder.build_script.defaults = true;
    builder.build_script.env_shader_spv_path.replace(true);
    let _ = builder.build()?;

    Ok(())
}
