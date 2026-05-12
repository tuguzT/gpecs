use std::{any, env, str::FromStr};

use gpecs::prelude::*;
use gpecs_ecs_benchmark::{cpu, gpu};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let context = &mut Context::new();
    let entity_count = parse_env("ENTITY_COUNT").unwrap_or(1_000_000);

    let cpu_repeat_count = parse_env("CPU_REPEAT_COUNT");
    let context = cpu::run(context, entity_count, cpu_repeat_count);
    context.destroy_all();

    let gpu_repeat_count = parse_env("GPU_REPEAT_COUNT");
    let context = gpu::run(context, entity_count, gpu_repeat_count);
    context.destroy_all();
}

fn parse_env<T>(key: &str) -> Option<T>
where
    T: FromStr,
{
    let Ok(value) = env::var(key).ok()?.parse() else {
        let type_name = any::type_name::<T>();
        panic!("`{key}` env should be `{type_name}`")
    };
    Some(value)
}
