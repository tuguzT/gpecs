use std::{any, borrow::Cow};

use crate::context::Context;

pub mod registry;
pub mod schedule;

mod impls;

pub trait System: 'static {
    fn run(&mut self, context: &mut Context);

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
    type Error<'ctx>;

    fn get_param(context: &mut Context) -> SystemParamResult<'_, Self>;
}
