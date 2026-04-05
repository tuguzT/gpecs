use core::iter::FusedIterator;

use gpecs_soa_erased::{
    CovariantFieldDescriptors,
    soa::field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuFieldDescriptors<T>
where
    T: ?Sized,
{
    next: usize,
    descriptors: T,
}

impl<T> From<T> for GpuFieldDescriptors<T> {
    fn from(descriptors: T) -> Self {
        Self {
            next: 0,
            descriptors,
        }
    }
}

impl<'a, T> FieldDescriptors<'a> for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + Clone,
{
    type Output = GpuFieldDescriptors<&'a [FieldDescriptor]>;

    fn field_descriptors(&'a self) -> Self::Output {
        let Self {
            ref descriptors,
            next,
        } = *self;

        let descriptors = descriptors.as_ref();
        GpuFieldDescriptors { next, descriptors }
    }
}

impl<T> CovariantFieldDescriptors for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + Clone,
{
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

impl<T> Iterator for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = FieldDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self {
            ref descriptors,
            ref mut next,
        } = *self;

        let index = *next + n;
        let descriptors = descriptors.as_ref();
        if index >= descriptors.len() {
            *next = descriptors.len();
            return None;
        }

        *next = index + 1;
        Some(descriptors[index])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<T> ExactSizeIterator for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn len(&self) -> usize {
        let Self {
            ref descriptors,
            next,
        } = *self;

        let descriptors = descriptors.as_ref();
        descriptors.len() - next
    }
}

impl<T> FusedIterator for GpuFieldDescriptors<T> where T: AsRef<[FieldDescriptor]> + ?Sized {}
