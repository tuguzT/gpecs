use std::{
    io::{self, Write},
    process::Command,
};

pub fn main() {
    const PATH: &str = env!("gpecs_spirv_example.spv");

    let output = Command::new("spirv-dis")
        .arg(PATH)
        .output()
        .expect("Failed to run spirv-dis");

    println!("Path to the shader: {PATH}\n");
    io::stdout().write_all(&output.stdout).unwrap();
}
