use std::error::Error;

use spirv_builder::{MetadataPrintout, SpirvBuilder, SpirvMetadata};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../gpecs_spirv_example", "spirv-unknown-spv1.3")
        .print_metadata(MetadataPrintout::Full)
        .spirv_metadata(SpirvMetadata::Full)
        .build()?;

    Ok(())
}
