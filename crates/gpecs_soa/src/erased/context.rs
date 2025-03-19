use alloc::boxed::Box;
use core::{
    alloc::Layout,
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
    mem,
};

use crate::traits::Soa;

use super::{assert::validate_layout, ErasedFieldMutPtr};

type ErasedDropFnParam<'a> = &'a [ErasedFieldMutPtr];
type ErasedDropFn = Box<dyn Fn(ErasedDropFnParam<'_>)>;

pub struct ErasedSoaContext<Fields> {
    pub(super) field_layouts: Box<[Layout]>,
    pub(super) drop_fields: Option<ErasedDropFn>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaContext<Fields> {
    #[inline]
    pub fn new<I, O>(field_layouts: I, drop_fields: O) -> Self
    where
        I: IntoIterator<Item: Borrow<Layout>>,
        O: Into<Option<ErasedDropFn>>,
    {
        Self {
            field_layouts: field_layouts
                .into_iter()
                .map(validate_layout::<Fields, _>)
                .collect(),
            drop_fields: drop_fields.into(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
        T::Context: 'static,
    {
        let field_layouts = T::field_layouts(&context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>)
            .collect();

        let drop_fields = move |data: ErasedDropFnParam<'_>| unsafe {
            let ptrs = data.iter().map(|ptr| ptr.as_ptr());
            let ptrs = T::ptrs_restore_mut(&context, ptrs);
            T::ptrs_drop_in_place(&context, ptrs);
        };
        let drop_fields: Option<ErasedDropFn> = if mem::needs_drop::<T::Fields>() {
            Some(Box::new(drop_fields))
        } else {
            None
        };

        Self {
            field_layouts,
            drop_fields,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn field_layouts(&self) -> &[Layout] {
        let Self { field_layouts, .. } = self;
        field_layouts.as_ref()
    }
}

impl<Fields> Debug for ErasedSoaContext<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { field_layouts, .. } = self;

        f.debug_struct("ErasedSoaContext")
            .field("field_layouts", field_layouts)
            .finish_non_exhaustive()
    }
}
