use std::{
    any,
    borrow::Cow,
    convert::Infallible,
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::{
    archetype::erased::error::ArchetypeError,
    bundle::Bundle,
    context::{Bundles, BundlesMut, Context},
    executor::cpu::system::registry::SystemId,
};

use super::{IntoSystem, System, SystemParam, SystemParamResult};

pub struct FnSystem<In, Fn> {
    f: Fn,
    phantom: PhantomData<fn() -> In>,
}

impl<In, Fn> FnSystem<In, Fn> {
    fn new(f: Fn) -> Self {
        let phantom = PhantomData;
        Self { f, phantom }
    }

    #[inline]
    pub fn fn_name() -> &'static str {
        any::type_name::<Fn>()
    }
}

impl<Fn> System for FnSystem<(), Fn>
where
    Fn: FnMut() + 'static,
{
    fn run(&mut self, _: SystemId, _: &mut Context) {
        let Self { f, .. } = self;
        f();
    }

    #[inline]
    fn name(&self) -> Cow<'static, str> {
        Self::fn_name().into()
    }
}

impl<Fn> System for FnSystem<SystemId, Fn>
where
    Fn: FnMut(SystemId) + 'static,
{
    fn run(&mut self, system_id: SystemId, _: &mut Context) {
        let Self { f, .. } = self;
        f(system_id);
    }

    #[inline]
    fn name(&self) -> Cow<'static, str> {
        Self::fn_name().into()
    }
}

impl<In, Fn> System for FnSystem<(In,), Fn>
where
    In: SystemParam + 'static,
    Fn: FnMut(In::Item<'_>) + 'static,
{
    fn run(&mut self, system_id: SystemId, context: &mut Context) {
        let Self { f, .. } = self;

        let Ok(param) = In::get_param(system_id, context) else {
            return;
        };
        f(param);
    }

    #[inline]
    fn name(&self) -> Cow<'static, str> {
        Self::fn_name().into()
    }
}

impl<In, Fn> System for FnSystem<(SystemId, In), Fn>
where
    In: SystemParam + 'static,
    Fn: FnMut(SystemId, In::Item<'_>) + 'static,
{
    fn run(&mut self, system_id: SystemId, context: &mut Context) {
        let Self { f, .. } = self;

        let Ok(param) = In::get_param(system_id, context) else {
            return;
        };
        f(system_id, param);
    }

    #[inline]
    fn name(&self) -> Cow<'static, str> {
        Self::fn_name().into()
    }
}

impl<Fn> IntoSystem<()> for Fn
where
    Fn: FnMut() + 'static,
{
    type System = FnSystem<(), Fn>;

    #[inline]
    fn into_system(self) -> Self::System {
        FnSystem::new(self)
    }
}

impl<Fn> IntoSystem<SystemId> for Fn
where
    Fn: FnMut(SystemId) + 'static,
{
    type System = FnSystem<SystemId, Fn>;

    #[inline]
    fn into_system(self) -> Self::System {
        FnSystem::new(self)
    }
}

impl<In, Fn> IntoSystem<In> for Fn
where
    In: SystemParam + 'static,
    Fn: FnMut(In) + FnMut(In::Item<'_>) + 'static,
{
    type System = FnSystem<(In,), Fn>;

    #[inline]
    fn into_system(self) -> Self::System {
        FnSystem::new(self)
    }
}

impl<In, Fn> IntoSystem<(SystemId, In)> for Fn
where
    In: SystemParam + 'static,
    Fn: FnMut(SystemId, In) + FnMut(SystemId, In::Item<'_>) + 'static,
{
    type System = FnSystem<(SystemId, In), Fn>;

    #[inline]
    fn into_system(self) -> Self::System {
        FnSystem::new(self)
    }
}

impl SystemParam for &Context {
    type Item<'ctx> = &'ctx Context;
    type Error<'ctx> = Infallible;

    #[inline]
    fn get_param(_: SystemId, context: &mut Context) -> SystemParamResult<'_, Self> {
        Ok(&*context)
    }
}

impl SystemParam for &mut Context {
    type Item<'ctx> = &'ctx mut Context;
    type Error<'ctx> = Infallible;

    #[inline]
    fn get_param(_: SystemId, context: &mut Context) -> SystemParamResult<'_, Self> {
        Ok(context)
    }
}

impl<B> SystemParam for Bundles<'_, B>
where
    B: Bundle,
{
    type Item<'ctx> = Bundles<'ctx, B>;
    type Error<'ctx> = ArchetypeError;

    #[inline]
    fn get_param(_: SystemId, context: &mut Context) -> SystemParamResult<'_, Self> {
        context.bundles::<B>()
    }
}

impl<B> SystemParam for BundlesMut<'_, B>
where
    B: Bundle,
{
    type Item<'ctx> = BundlesMut<'ctx, B>;
    type Error<'ctx> = ArchetypeError;

    #[inline]
    fn get_param(_: SystemId, context: &mut Context) -> SystemParamResult<'_, Self> {
        context.bundles_mut::<B>()
    }
}

impl Debug for dyn System {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        f.debug_struct("System")
            .field("name", &name)
            .finish_non_exhaustive()
    }
}
