use std::{
    any,
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::context::Context;

use super::{IntoSystem, System};

pub struct FnSystem<I, F> {
    f: F,
    phantom: PhantomData<fn() -> I>,
}

impl<F> System for FnSystem<(), F>
where
    F: FnMut() + 'static,
{
    fn run(&mut self, _: &mut Context) {
        let Self { f, .. } = self;
        f()
    }

    #[inline]
    fn name(&self) -> std::borrow::Cow<'static, str> {
        any::type_name::<F>().into()
    }
}

impl<F> IntoSystem<()> for F
where
    F: FnMut() + 'static,
{
    type System = FnSystem<(), F>;

    #[inline]
    fn into_system(self) -> Self::System {
        FnSystem {
            f: self,
            phantom: PhantomData,
        }
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
