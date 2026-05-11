use std::{
    borrow::{Borrow, Cow},
    time::Duration,
};

use gpecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatisticsRecord {
    pub system: u32,
    pub name: Cow<'static, str>,
    pub archetype: ArchetypeId,
    pub elapsed: Duration,
}

pub fn log_statistics<I>(group: &str, statistics: I, index: u128, elapsed: Duration)
where
    I: IntoIterator<Item: Borrow<StatisticsRecord>>,
{
    for record in statistics {
        let StatisticsRecord {
            system,
            name,
            archetype,
            elapsed,
        } = record.borrow();
        log::info!(">>>> {group} system {system} `{name}` with {archetype} took {elapsed:?}");
    }
    log::info!(">>! Execution of all the {group} systems {index} took {elapsed:?}");
}
