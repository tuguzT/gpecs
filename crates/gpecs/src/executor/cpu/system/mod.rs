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

pub trait IntoSystem<I> {
    type System: System;

    fn into_system(self) -> Self::System;
}
