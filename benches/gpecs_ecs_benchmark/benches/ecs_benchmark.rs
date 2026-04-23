use gpecs::prelude::*;
use gpecs_ecs_benchmark::{cpu, gpu};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut context = Context::new();
    cpu::run(&mut context);
    gpu::run(&mut context);
}
