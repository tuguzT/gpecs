use std::{
    fs::{self, File},
    io::Write,
    iter,
    path::Path,
    time::Duration,
};

use crate::statistics::StatisticsRecord;

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
    let path = format!("./dump/{group}/statistics-{entity_count}.csv");
    let path = Path::new(&path);

    let prefix = path.parent().expect("path should have a parent directory");
    fs::create_dir_all(prefix)?;

    let file = File::create(path)?;
    let writer = csv::WriterBuilder::new().from_writer(file);
    Ok(writer)
}

pub fn dump_csv_header<W>(record: &CsvRecord, writer: &mut csv::Writer<W>) -> csv::Result<()>
where
    W: Write,
{
    fn record_header(record: &StatisticsRecord) -> String {
        let StatisticsRecord {
            system, archetype, ..
        } = record;
        format!("system {system} {archetype}")
    }

    let CsvRecord { statistics, .. } = record;

    let record = statistics
        .iter()
        .map(record_header)
        .chain(iter::once("total".into()));
    writer.write_record(record)
}

pub fn dump_csv_record<W>(record: CsvRecord, writer: &mut csv::Writer<W>) -> csv::Result<()>
where
    W: Write,
{
    let CsvRecord {
        elapsed,
        statistics,
    } = record;

    let record = statistics
        .iter()
        .map(|statistics| statistics.elapsed.as_secs_f64().to_string())
        .chain(iter::once(elapsed.as_secs_f64().to_string()));
    writer.write_record(record)
}
