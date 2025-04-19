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

pub type SystemParamResult<'context, T> =
    Result<<T as SystemParam>::Item<'context>, <T as SystemParam>::Error<'context>>;

pub trait SystemParam: Sized {
    type Item<'context>: SystemParam;
    type Error<'context>;

    fn get_param(context: &mut Context) -> SystemParamResult<Self>;
}
