use gpecs::prelude::*;

mod cpu;
mod dump;
mod gpu;
mod setup;
mod statistics;

const ENTITY_COUNT: u32 = if cfg!(debug_assertions) {
    2_400
} else {
    1_200_000
};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let context = &mut Context::new();

    let context = cpu::run(context, ENTITY_COUNT, Some(100));
    context.destroy_all();

    let context = gpu::run(context, ENTITY_COUNT, Some(100));
    context.destroy_all();
}
