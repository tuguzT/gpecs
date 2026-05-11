use std::{
    fs::{self, File},
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

pub fn dump_csv_records_into_file<I>(records: I, group: &str, entity_count: u32)
where
    I: IntoIterator<Item = CsvRecord>,
{
    let path = format!("./dump/{group}/statistics-{entity_count}.csv");
    let path = Path::new(&path);

    let prefix = path.parent().expect("failed to get parent directory");
    fs::create_dir_all(prefix).expect("failed to create parent directory");

    let file = File::create(path).expect("failed to create csv file");
    let mut writer = csv::WriterBuilder::new().from_writer(file);

    let mut records = records.into_iter().peekable();
    if let Some(record) = records.peek() {
        let CsvRecord { statistics, .. } = record;

        let record = statistics
            .iter()
            .map(record_header)
            .chain(iter::once("total".into()));
        writer
            .write_record(record)
            .expect("csv header should be saved into a file");
    }
    for record in records {
        let CsvRecord {
            elapsed,
            statistics,
        } = record;

        let record = statistics
            .iter()
            .map(|statistics| statistics.elapsed.as_secs_f64().to_string())
            .chain(iter::once(elapsed.as_secs_f64().to_string()));
        writer
            .write_record(record)
            .expect("csv record should be saved into a file");
    }

    writer.flush().expect("csv file data should be saved");
}

fn record_header(record: &StatisticsRecord) -> String {
    let StatisticsRecord {
        system, archetype, ..
    } = record;
    format!("system {system} {archetype}")
}
