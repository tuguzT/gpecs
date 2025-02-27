use std::{any::Any, collections::BTreeSet};

use gpecs_sparse::set::EpochSparseSet;

use crate::{component::ComponentId, id::Id, soa::Soa};

pub struct Archetype {
    #[allow(dead_code)]
    component_ids: BTreeSet<ComponentId>,
    erased_storage: Box<dyn ErasedStorage>,
}

impl Archetype {
    pub fn entities(&self) -> &[Id] {
        let Self { erased_storage, .. } = self;
        erased_storage.entities()
    }

    #[allow(dead_code)]
    fn insert_internal(&mut self, id: Id, value: Box<dyn Any>) {
        let Self { erased_storage, .. } = self;
        erased_storage
            .insert(id, value)
            .expect("type of value should match with storage type");
    }

    #[allow(dead_code)]
    fn remove_internal(&mut self, id: Id) -> Option<Box<dyn Any>> {
        let Self { erased_storage, .. } = self;
        erased_storage.remove(id)
    }
}

trait ErasedStorage {
    fn entities(&self) -> &[Id];

    fn insert(&mut self, id: Id, value: Box<dyn Any>) -> Result<(), Box<dyn Any>>;

    fn remove(&mut self, id: Id) -> Option<Box<dyn Any>>;
}

impl<V> ErasedStorage for EpochSparseSet<Id, V>
where
    V: Soa + 'static,
{
    fn entities(&self) -> &[Id] {
        self.as_keys_slice()
    }

    fn insert(&mut self, id: Id, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        let value = value.downcast::<V>()?;
        self.insert(id, *value);
        Ok(())
    }

    fn remove(&mut self, id: Id) -> Option<Box<dyn Any>> {
        let value = self.remove(id)?;
        let value = Box::new(value);
        Some(value)
    }
}
