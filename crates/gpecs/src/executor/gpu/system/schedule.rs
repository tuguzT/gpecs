use std::iter::FusedIterator;

use indexmap::IndexSet;

use super::registry::GpuSystemId;

#[derive(Debug, Default)]
pub struct GpuSystemSchedule {
    systems: IndexSet<GpuSystemId>,
}

impl GpuSystemSchedule {
    #[inline]
    pub fn new() -> Self {
        Self {
            systems: IndexSet::new(),
        }
    }

    #[inline]
    pub fn add_system(&mut self, system: GpuSystemId) -> bool {
        let Self { systems } = self;
        systems.insert(system)
    }

    #[inline]
    pub fn remove_system(&mut self, system: GpuSystemId) -> bool {
        let Self { systems } = self;
        systems.shift_remove(&system)
    }

    #[inline]
    // TODO: add specific iterator type
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = GpuSystemId> + ExactSizeIterator + FusedIterator + '_ {
        let Self { systems } = self;
        systems.iter().copied()
    }
}
