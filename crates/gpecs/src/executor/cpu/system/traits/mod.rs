use std::{any, borrow::Cow, error::Error};

use crate::{context::Context, executor::cpu::system::registry::SystemId};

mod impls;

pub trait System: 'static {
    fn run(&mut self, system_id: SystemId, context: &mut Context);

    #[inline]
    fn name(&self) -> Cow<'static, str> {
        any::type_name::<Self>().into()
    }
}

pub trait IntoSystem<In>: Sized {
    type System: System;

    fn into_system(self) -> Self::System;
}

pub type SystemParamResult<'ctx, T> =
    Result<<T as SystemParam>::Item<'ctx>, <T as SystemParam>::Error<'ctx>>;

pub trait SystemParam: Sized {
    type Item<'ctx>: SystemParam;
    type Error<'ctx>: Error;

    fn get_param(system_id: SystemId, context: &mut Context) -> SystemParamResult<'_, Self>;
}
