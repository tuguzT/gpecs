use gpecs::prelude::*;
use gpecs_ecs_benchmark::{cpu, gpu};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let entity_count = std::env::var("ENTITY_COUNT")
        .map(|v| v.parse().expect("`ENTITY_COUNT` env should contain `u32`"))
        .unwrap_or(1_000_000);

    let context = &mut Context::new();

    let context = cpu::run(context, entity_count, Some(12));
    context.destroy_all();

    let context = gpu::run(context, entity_count, Some(12));
    context.destroy_all();
}
