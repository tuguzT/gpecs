use std::{any, borrow::Cow};

use crate::context::Context;

pub mod registry;

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

pub trait SystemParam: Sized {
    type Item<'context>: SystemParam;

    fn get_param<'context>(context: &'context mut Context) -> Self::Item<'context>;
}
