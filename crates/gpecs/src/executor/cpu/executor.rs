use crate::context::Context;

use super::system::{
    IntoSystem,
    registry::{SystemId, SystemRegistry},
    schedule::SystemSchedule,
};

#[derive(Debug)]
pub struct CpuExecutor<'ctx> {
    context: &'ctx mut Context,
    systems: SystemRegistry,
    schedule: SystemSchedule,
}

impl<'ctx> CpuExecutor<'ctx> {
    #[inline]
    pub fn new(context: &'ctx mut Context) -> Self {
        Self {
            context,
            systems: SystemRegistry::new(),
            schedule: SystemSchedule::new(),
        }
    }

    #[inline]
    pub fn context(&self) -> &Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn context_mut(&mut self) -> &mut Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn into_context(self) -> &'ctx mut Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn register_system<In, S>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<In>,
    {
        let Self { systems, .. } = self;
        systems.register_system(system)
    }

    #[inline]
    pub fn add_system(&mut self, system: SystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.add_system(system)
    }

    #[inline]
    pub fn remove_system(&mut self, system: SystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.remove_system(system)
    }

    #[inline]
    pub fn execute(&mut self) {
        let Self {
            ref mut context,
            ref mut systems,
            ref schedule,
        } = *self;

        schedule.iter().for_each(|system_id| {
            let Some(system) = systems.get_mut_system(system_id) else {
                unreachable!("{system_id} should be present");
            };
            system.run(context);
        });
    }
}
