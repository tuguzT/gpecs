use core::{alloc::Layout, iter};

use crate::soa::{
    field::{FieldLayouts, IntoFieldLayouts, IntoFieldLayoutsIter},
    layout::WithLayout,
    traits::RawSoa,
};

#[derive(Debug, Clone, Copy)]
pub struct DenseFieldLayouts<T>
where
    T: ?Sized,
{
    key: Layout,
    values: T,
}

impl<T> DenseFieldLayouts<T> {
    #[inline]
    pub fn new<'a, K, V>(context: &'a V::Context) -> Self
    where
        V: RawSoa + ?Sized,
        V::Context: FieldLayouts<'a, V, Output = T>,
    {
        let key = Layout::new::<K>();
        let values = context.field_layouts();
        Self { key, values }
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, T) {
        let Self { key, values } = self;
        (key, values)
    }
}

impl<T> IntoIterator for DenseFieldLayouts<T>
where
    T: IntoIterator<Item: WithLayout>,
{
    type Item = Layout;
    type IntoIter = iter::Chain<iter::Once<Layout>, IntoFieldLayouts<T::IntoIter>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values } = self;

        let values = values.into_field_layouts();
        iter::once(key).chain(values)
    }
}
