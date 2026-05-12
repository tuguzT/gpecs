use std::{any, env, str::FromStr};

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

    let context = &mut Context::new();
    let entity_count = parse_env("ENTITY_COUNT").unwrap_or(DEFAULT_ENTITY_COUNT);

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
