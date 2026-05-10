use core::{
    alloc::Layout,
    iter::{self, Chain, Once},
};

use gpecs_soa::{
    field::{FieldLayouts, IntoFieldLayouts, IntoFieldLayoutsIter},
    layout::WithLayout,
    traits::RawSoa,
};

#[derive(Debug, Clone, Copy)]
pub struct KeyValueFieldLayouts<T>
where
    T: ?Sized,
{
    key: Layout,
    values: T,
}

impl<T> KeyValueFieldLayouts<T> {
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

impl<T> IntoIterator for KeyValueFieldLayouts<T>
where
    T: IntoIterator<Item: WithLayout>,
{
    type Item = Layout;
    type IntoIter = Chain<Once<Layout>, IntoFieldLayouts<T::IntoIter>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { key, values } = self;

        let values = values.into_field_layouts();
        iter::once(key).chain(values)
    }
}
