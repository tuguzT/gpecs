use std::error::Error;

use spirv_builder::{MetadataPrintout, ShaderPanicStrategy, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../gpecs_spirv_example", "spirv-unknown-spv1.3")
        .print_metadata(MetadataPrintout::Full)
        .extension("SPV_KHR_non_semantic_info")
        .shader_panic_strategy(ShaderPanicStrategy::DebugPrintfThenExit {
            print_inputs: true,
            print_backtrace: true,
        })
        .build()?;

    Ok(())
}
