use std::{borrow::Cow, time::Duration};

use gpecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatisticsRecord {
    pub system: u32,
    pub name: Cow<'static, str>,
    pub archetype: ArchetypeId,
    pub elapsed: Duration,
}
