use std::{env, error::Error, ffi::OsStr, str::FromStr};

use gpecs::prelude::*;
use gpecs_ecs_benchmark::{dump::*, logs::log_statistics};
use gpecs_ecs_benchmark_core::{cpu, gpu};
use gpecs_ecs_benchmark_types::{components::NONE_SPRITE, framebuffer::Framebuffer};

#[cfg(feature = "dhat")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::builder().init();

    let context = &mut Context::new();
    let entity_count = parse_env("ENTITY_COUNT")?.unwrap_or(1_000_000);

    #[cfg(feature = "dhat")]
    let _profiler = dhat::Profiler::builder()
        .file_name(format!("dump/_dhat-{entity_count}.json"))
        .build();

    let framebuffer_width = parse_env("FRAMEBUFFER_WIDTH")?.unwrap_or(320);
    let framebuffer_height = parse_env("FRAMEBUFFER_HEIGHT")?.unwrap_or(240);
    let framebuffer_size = (framebuffer_width * framebuffer_height).try_into()?;
    let framebuffer = Framebuffer::new(
        framebuffer_width,
        framebuffer_height,
        vec![NONE_SPRITE; framebuffer_size],
    );
    let spawn_area_margin = parse_env("SPAWN_AREA_MARGIN")?.unwrap_or(100);

    let cpu_repeat_count = parse_env("CPU_REPEAT_COUNT")?;
    let mut csv_writer = create_csv_writer("cpu", entity_count)?;
    let f = |i, elapsed, statistics: Vec<_>, framebuffer: &Framebuffer<_>| -> csv::Result<()> {
        log_statistics("CPU", statistics.as_slice(), i, elapsed);

        log::info!(">>> Saving framebuffer state {i} to file...");
        dump_framebuffer_into_file(framebuffer, "cpu", i, entity_count)?;

        let csv_record = CsvRecord::new(elapsed, statistics);
        dump_csv_record(csv_record, i == 0, &mut csv_writer)?;

        csv_writer.flush()?;
        Ok(())
    };
    let context = cpu::run(
        context,
        entity_count,
        cpu_repeat_count,
        framebuffer.clone(),
        spawn_area_margin,
        f,
    )?;
    context.destroy_all();

    let (device, queue) = gpu::init_wgpu();
    let device_name = device.adapter_info().name;

    let gpu_repeat_count = parse_env("GPU_REPEAT_COUNT")?;
    let group = &format!("gpu-{}", device_name.to_lowercase().replace(' ', "-"));
    let mut csv_writer = create_csv_writer(group, entity_count)?;

    let log_group = &format!("GPU `{device_name}`");
    let f = |i, elapsed, statistics: Vec<_>, framebuffer: &Framebuffer<_>| -> csv::Result<()> {
        log_statistics(log_group, statistics.as_slice(), i, elapsed);

        log::info!(">>> Saving framebuffer state {i} to file...");
        dump_framebuffer_into_file(framebuffer, group, i, entity_count)?;

        let csv_record = CsvRecord::new(elapsed, statistics);
        dump_csv_record(csv_record, i == 0, &mut csv_writer)?;

        csv_writer.flush()?;
        Ok(())
    };
    let context = gpu::run(
        &device,
        &queue,
        context,
        entity_count,
        gpu_repeat_count,
        framebuffer,
        spawn_area_margin,
        f,
    )?;
    context.destroy_all();

    Ok(())
}

fn parse_env<T>(key: impl AsRef<OsStr>) -> Result<Option<T>, T::Err>
where
    T: FromStr,
{
    env::var(key).ok().as_deref().map(str::parse).transpose()
}
