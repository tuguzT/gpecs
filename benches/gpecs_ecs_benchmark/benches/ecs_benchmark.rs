use gpecs::prelude::*;
use gpecs_ecs_benchmark::{cpu, gpu};

const ENTITY_COUNT: u32 = 1_000_000;

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let context = &mut Context::new();

    let context = cpu::run(context, ENTITY_COUNT, Some(12));
    context.destroy_all();

    let context = gpu::run(context, ENTITY_COUNT, Some(12));
    context.destroy_all();
}
