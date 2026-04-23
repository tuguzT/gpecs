use gpecs::prelude::*;

mod cpu;
mod gpu;
mod setup;

const ITER_COUNT: usize = 25;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut context = Context::new();
    cpu::run(&mut context);
    gpu::run(&mut context);
}
