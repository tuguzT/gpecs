use std::ops::Deref;

use arrayvec::{ArrayVec, IntoIter};
use gpecs_soa::{
    field::{FieldLayouts, FieldLayoutsOutput},
    layout::WithLayout,
};
use gpecs_soa_erased::CovariantFieldLayouts;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ArrayLayouts<T, const CAP: usize>(pub ArrayVec<T, CAP>);

impl<T, const CAP: usize> Default for ArrayLayouts<T, CAP> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, const CAP: usize> Deref for ArrayLayouts<T, CAP> {
    type Target = ArrayVec<T, CAP>;

    fn deref(&self) -> &Self::Target {
        let Self(array_vec) = self;
        array_vec
    }
}

impl<T, const CAP: usize> IntoIterator for ArrayLayouts<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;

    fn into_iter(self) -> Self::IntoIter {
        let Self(array_vec) = self;
        array_vec.into_iter()
    }
}

impl<A, T, const CAP: usize> FromIterator<A> for ArrayLayouts<T, CAP>
where
    T: From<A>,
{
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let array_vec = iter.into_iter().map(From::from).collect();
        Self(array_vec)
    }
}

impl<A, T, const CAP: usize> Extend<A> for ArrayLayouts<T, CAP>
where
    T: From<A>,
{
    fn extend<I: IntoIterator<Item = A>>(&mut self, iter: I) {
        let Self(array_vec) = self;
        array_vec.extend(iter.into_iter().map(From::from));
    }
}

impl<'a, T, const CAP: usize> FieldLayouts<'a> for ArrayLayouts<T, CAP>
where
    T: WithLayout + 'a,
{
    type Output = &'a [T];

    fn field_layouts(&'a self) -> Self::Output {
        self
    }
}

impl<T, const CAP: usize> CovariantFieldLayouts for ArrayLayouts<T, CAP>
where
    T: WithLayout + 'static,
{
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}
