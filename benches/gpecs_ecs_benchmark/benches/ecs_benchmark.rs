use std::{any, env, str::FromStr};

use gpecs::prelude::*;
use gpecs_ecs_benchmark::{cpu, gpu};
use gpecs_ecs_benchmark_types::{components::NONE_SPRITE, framebuffer::Framebuffer};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let context = &mut Context::new();
    let entity_count = parse_env("ENTITY_COUNT").unwrap_or(1_000_000);

    let framebuffer_width = parse_env("FRAMEBUFFER_WIDTH").unwrap_or(320);
    let framebuffer_height = parse_env("FRAMEBUFFER_HEIGHT").unwrap_or(240);
    let framebuffer_size = (framebuffer_width * framebuffer_height)
        .try_into()
        .expect("framebuffer size should fit into `usize`");
    let framebuffer = Framebuffer::new(
        framebuffer_width,
        framebuffer_height,
        vec![NONE_SPRITE; framebuffer_size],
    );
    let spawn_area_margin = parse_env("SPAWN_AREA_MARGIN").unwrap_or(100);

    let cpu_repeat_count = parse_env("CPU_REPEAT_COUNT");
    let context = cpu::run(
        context,
        entity_count,
        cpu_repeat_count,
        framebuffer.clone(),
        spawn_area_margin,
    );
    context.destroy_all();

    let gpu_repeat_count = parse_env("GPU_REPEAT_COUNT");
    let context = gpu::run(
        context,
        entity_count,
        gpu_repeat_count,
        framebuffer,
        spawn_area_margin,
    );
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
