use gpecs::prelude::*;

mod cpu;
mod dump;
mod gpu;
mod setup;
mod statistics;

const DEFAULT_ENTITY_COUNT: u32 = if cfg!(debug_assertions) {
    2_400
} else {
    1_200_000
};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let entity_count = std::env::var("ENTITY_COUNT")
        .map(|v| v.parse().expect("`ENTITY_COUNT` env should contain `u32`"))
        .unwrap_or(DEFAULT_ENTITY_COUNT);

    let context = &mut Context::new();

    let context = cpu::run(context, entity_count, Some(100));
    context.destroy_all();

    let context = gpu::run(context, entity_count, Some(100));
    context.destroy_all();
}
