use std::{env, error::Error, ffi::OsStr, str::FromStr};

use gpecs::prelude::*;

use gpecs_simple_core::{cpu, gpu};

use self::{
    dump::{CsvRecord, create_csv_writer, dump_csv_record},
    logs::log_statistics,
};

mod dump;
mod logs;

const DEFAULT_ENTITY_COUNT: u32 = if cfg!(debug_assertions) {
    2_400
} else {
    1_200_000
};

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::builder().try_init()?;

    let context = &mut Context::new();
    let entity_count = parse_env("ENTITY_COUNT")?.unwrap_or(DEFAULT_ENTITY_COUNT);

    let cpu_repeat_count = parse_env("CPU_REPEAT_COUNT")?;
    let mut csv_writer = create_csv_writer("cpu", entity_count)?;
    let f = |i, elapsed, statistics: Vec<_>| -> csv::Result<()> {
        log_statistics("CPU", statistics.as_slice(), i, elapsed);

        let csv_record = CsvRecord::new(elapsed, statistics);
        dump_csv_record(csv_record, i == 0, &mut csv_writer)?;

        csv_writer.flush()?;
        Ok(())
    };
    let context = cpu::run(context, entity_count, cpu_repeat_count, f)?;
    context.destroy_all();

    let (device, queue) = gpu::init_wgpu();
    let device_name = device.adapter_info().name;

    let gpu_repeat_count = parse_env("GPU_REPEAT_COUNT")?;
    let group = format!("gpu-{}", device_name.to_lowercase().replace(' ', "-"));
    let mut csv_writer = create_csv_writer(group, entity_count)?;

    let group = &format!("GPU `{device_name}`");
    let f = |i, elapsed, statistics: Vec<_>| -> csv::Result<()> {
        log_statistics(group, statistics.as_slice(), i, elapsed);

        let csv_record = CsvRecord::new(elapsed, statistics);
        dump_csv_record(csv_record, i == 0, &mut csv_writer)?;

        csv_writer.flush()?;
        Ok(())
    };
    let context = gpu::run(&device, &queue, context, entity_count, gpu_repeat_count, f)?;
    context.destroy_all();

    Ok(())
}

fn parse_env<T>(key: impl AsRef<OsStr>) -> Result<Option<T>, T::Err>
where
    T: FromStr,
{
    env::var(key).ok().as_deref().map(str::parse).transpose()
}
