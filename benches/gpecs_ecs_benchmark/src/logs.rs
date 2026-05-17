use std::{borrow::Borrow, time::Duration};

use gpecs_ecs_benchmark_core::statistics::StatisticsRecord;

pub fn log_statistics<I>(group: impl AsRef<str>, statistics: I, i: u128, elapsed: Duration)
where
    I: IntoIterator<Item: Borrow<StatisticsRecord>>,
{
    let group = group.as_ref();
    for record in statistics {
        let StatisticsRecord {
            system,
            name,
            archetype,
            elapsed,
        } = record.borrow();
        log::info!(">>>> {group} system {system} `{name}` with {archetype} took {elapsed:?}");
    }
    log::info!(">>! Execution {i} of all the {group} systems took {elapsed:?}");
}
