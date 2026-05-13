use std::{
    fs::{self, File},
    io::Write,
    iter,
    path::Path,
    time::Duration,
};

use gpecs_simple_core::statistics::StatisticsRecord;

pub struct CsvRecord {
    elapsed: Duration,
    statistics: Vec<StatisticsRecord>,
}

impl CsvRecord {
    pub fn new<I>(elapsed: Duration, statistics: I) -> Self
    where
        I: IntoIterator<Item = StatisticsRecord>,
    {
        let statistics = statistics.into_iter().collect();
        Self {
            elapsed,
            statistics,
        }
    }
}

pub fn create_csv_writer(group: &str, entity_count: u32) -> csv::Result<csv::Writer<impl Write>> {
    let path = format!("./dump/{group}-{entity_count}-statistics.csv");
    let path = Path::new(&path);

    let prefix = path.parent().expect("path should have a parent directory");
    fs::create_dir_all(prefix)?;

    let file = File::create(path)?;
    let writer = csv::WriterBuilder::new().from_writer(file);
    Ok(writer)
}

pub fn dump_csv_record<W>(
    record: CsvRecord,
    with_header: bool,
    writer: &mut csv::Writer<W>,
) -> csv::Result<()>
where
    W: Write,
{
    let CsvRecord {
        statistics,
        elapsed,
    } = record;

    if with_header {
        let record = statistics
            .iter()
            .map(record_header)
            .chain(iter::once("total".into()));
        writer.write_record(record)?;
    }

    let record = statistics
        .iter()
        .map(|statistics| record_elapsed(statistics.elapsed))
        .chain(iter::once(record_elapsed(elapsed)));
    writer.write_record(record)
}

fn record_header(record: &StatisticsRecord) -> String {
    let StatisticsRecord {
        system, archetype, ..
    } = record;
    format!("system {system} {archetype}")
}

fn record_elapsed(elapsed: Duration) -> String {
    elapsed.as_secs_f64().to_string()
}
