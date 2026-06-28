use crate::context::Context;

use super::system::{
    IntoSystem,
    registry::{SystemId, SystemRegistry},
    schedule::SystemSchedule,
};

#[derive(Debug)]
pub struct CpuExecutor<T>
where
    T: ?Sized,
{
    systems: SystemRegistry,
    schedule: SystemSchedule,
    context: T,
}

impl<T> CpuExecutor<T> {
    #[inline]
    pub fn new(context: T) -> Self {
        Self {
            context,
            systems: SystemRegistry::new(),
            schedule: SystemSchedule::new(),
        }
    }

    #[inline]
    pub fn into_context(self) -> T {
        let Self { context, .. } = self;
        context
    }
}

impl<T> CpuExecutor<T>
where
    T: ?Sized,
{
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
}

impl<T> CpuExecutor<T>
where
    T: AsRef<Context> + ?Sized,
{
    #[inline]
    pub fn context(&self) -> &Context {
        let Self { context, .. } = self;
        context.as_ref()
    }
}

impl<T> CpuExecutor<T>
where
    T: AsMut<Context> + ?Sized,
{
    #[inline]
    pub fn context_mut(&mut self) -> &mut Context {
        let Self { context, .. } = self;
        context.as_mut()
    }

    #[inline]
    pub fn execute(&mut self) {
        let Self {
            ref mut context,
            ref mut systems,
            ref schedule,
        } = *self;

        let context = context.as_mut();
        schedule.iter().for_each(|system_id| {
            let Some(system) = systems.get_mut_system(system_id) else {
                unreachable!("{system_id} should be present");
            };
            system.run(system_id, context);
        });
    }
}
