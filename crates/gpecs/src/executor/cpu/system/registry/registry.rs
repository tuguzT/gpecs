#![expect(clippy::module_inception)]

use crate::executor::cpu::system::{
    IntoSystem, System,
    registry::{
        SystemId, SystemIds,
        id::{system_id_from_usize, system_id_into_usize},
    },
};

#[derive(Debug, Default)]
pub struct SystemRegistry {
    systems: Vec<Box<dyn System>>,
}

impl SystemRegistry {
    #[inline]
    pub fn new() -> Self {
        let systems = Vec::new();
        Self { systems }
    }

    #[inline]
    pub fn register_system<S, In>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<In>,
    {
        let Self { systems } = self;

        let index = systems.len();
        let id = system_id_from_usize(index);

        let system = Box::new(system.into_system());
        systems.push(system);

        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { systems } = self;
        systems.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { systems } = self;
        systems.is_empty()
    }

    #[inline]
    pub fn get_system(&self, system_id: SystemId) -> Option<&dyn System> {
        let Self { systems } = self;
        systems
            .get(system_id_into_usize(system_id))
            .map(AsRef::as_ref)
    }

    #[inline]
    pub fn get_mut_system(&mut self, system_id: SystemId) -> Option<&mut dyn System> {
        let Self { systems } = self;
        systems
            .get_mut(system_id_into_usize(system_id))
            .map(AsMut::as_mut)
    }

    #[inline]
    pub fn system_ids(&self) -> SystemIds {
        let index = self.len();
        let len = system_id_from_usize(index).into_u32();
        SystemIds::new(0..len)
    }
}
