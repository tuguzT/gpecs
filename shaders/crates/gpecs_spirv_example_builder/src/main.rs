use std::{error::Error, fs};

use gpecs_spirv_analysis::Work;
use rspirv::{binary::Parser, dr::Loader};

pub fn main() -> Result<(), Box<dyn Error>> {
    const PATH: &str = env!("gpecs_spirv_example.spv");

    let mut loader = Loader::new();

    let file_data = fs::read(PATH)?;
    let parser = Parser::new(&file_data, &mut loader);
    parser.parse()?;

    let module = loader.module();
    println!("\nCalculated work: {:?}", module.work()?);

    Ok(())
}
