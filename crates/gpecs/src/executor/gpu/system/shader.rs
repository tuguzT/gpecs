use std::{iter::FusedIterator, num::NonZeroU64};

use indexmap::{IndexMap, IndexSet, map};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ComputePipeline, ComputePipelineDescriptor, Device,
    PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, ShaderModule,
    ShaderStages,
};

use crate::{
    archetype::error::DuplicateComponentError,
    component::{registry::ComponentRegistry, utils::try_collect_component_ids},
    entity::Entity,
    executor::gpu::component::registry::GpuComponentId,
};

use super::registry::{GpuSystemDescriptor, GpuSystemId};

#[derive(Debug)]
pub struct SystemShader {
    entity_entry: Option<BindGroupLayoutEntry>,
    component_entries: IndexMap<GpuComponentId, Option<BindGroupLayoutEntry>>,
    additional_entries: Vec<BindGroupLayoutEntry>,
    shader_module: ShaderModule,
    workgroup_count: Option<u32>,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    compute_pipeline: ComputePipeline,
}

impl SystemShader {
    #[inline]
    pub(super) fn new<C, B>(
        components: &ComponentRegistry,
        gpu_device: &Device,
        system_id: GpuSystemId,
        descriptor: GpuSystemDescriptor<C, B>,
    ) -> Result<Self, DuplicateComponentError>
    where
        C: IntoIterator<Item = GpuComponentId>,
        B: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        let GpuSystemDescriptor {
            shader_module,
            entry_point,
            workgroup_count,
            bind_components,
            bind_entities,
            additional_bindings,
        } = descriptor;

        let component_ids = bind_components.into_iter().map(Into::into);
        let component_ids = try_collect_component_ids(component_ids, IndexSet::<_>::insert)?;
        let component_ids: IndexSet<_> = component_ids
            .into_iter()
            .map(|id| unsafe { GpuComponentId::from_id(id) })
            .collect();

        let additional_entries: Vec<_> = additional_bindings.into_iter().collect();

        let max_entries = component_ids.len() + additional_entries.len() + (bind_entities as usize);
        let mut entries = Vec::with_capacity(max_entries);

        let mut entity_entry = None;
        if bind_entities {
            let entry = BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    min_binding_size: Some(
                        u64::try_from(size_of::<Entity>())
                            .expect("size of `Entity` should fit in `u64`")
                            .try_into()
                            .expect("size of `Entity` cannot be zero"),
                    ),
                    has_dynamic_offset: false,
                },
                count: None,
            };
            entity_entry = entry.into();
            entries.push(entry);
        }

        let mut component_entries = IndexMap::with_capacity(max_entries);
        for (index, &component_id) in component_ids.iter().enumerate() {
            let Some(info) = components.get_component_info(component_id.into()) else {
                unreachable!("component {component_id:?} should exist");
            };
            let size_of_component = info
                .descriptor()
                .layout()
                .size()
                .try_into()
                .expect("size of component should fit in `u64`");
            let Some(min_binding_size) = NonZeroU64::new(size_of_component) else {
                component_entries.insert(component_id, None);
                continue;
            };

            let component_entry = BindGroupLayoutEntry {
                binding: (index + (bind_entities as usize))
                    .try_into()
                    .expect("count of bindings should fit in `u32`"),
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    min_binding_size: Some(min_binding_size),
                    has_dynamic_offset: false,
                },
                count: None,
            };
            if component_entries
                .insert(component_id, Some(component_entry))
                .is_some()
            {
                unreachable!("duplicate component {component_id:?} in shader {system_id:?}");
            };
            entries.push(component_entry);
        }

        entries.extend(additional_entries.iter().copied());

        let bind_group_layout_label = format!("`gpecs` {system_id:?} bind group layout");
        let bind_group_layout_desc = BindGroupLayoutDescriptor {
            label: Some(&bind_group_layout_label),
            entries: &entries,
        };
        let bind_group_layout = gpu_device.create_bind_group_layout(&bind_group_layout_desc);

        let pipeline_layout_label = format!("`gpecs` {system_id:?} pipeline layout");
        let pipeline_layout_desc = PipelineLayoutDescriptor {
            label: Some(&pipeline_layout_label),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        };
        let pipeline_layout = gpu_device.create_pipeline_layout(&pipeline_layout_desc);

        let compute_pipeline_label = format!("`gpecs` {system_id:?} compute pipeline");
        let compute_pipeline_desc = ComputePipelineDescriptor {
            label: Some(&compute_pipeline_label),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point,
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        };
        let compute_pipeline = gpu_device.create_compute_pipeline(&compute_pipeline_desc);

        Ok(Self {
            entity_entry,
            component_entries,
            additional_entries,
            workgroup_count,
            shader_module,
            bind_group_layout,
            pipeline_layout,
            compute_pipeline,
        })
    }

    #[inline]
    pub fn shader_module(&self) -> &ShaderModule {
        let Self { shader_module, .. } = self;
        shader_module
    }

    #[inline]
    pub fn workgroup_count(&self) -> Option<u32> {
        let Self {
            workgroup_count, ..
        } = *self;
        workgroup_count
    }

    #[inline]
    pub fn entity_entry(&self) -> Option<&BindGroupLayoutEntry> {
        let Self { entity_entry, .. } = self;
        entity_entry.as_ref()
    }

    #[inline]
    pub fn component_entries(&self) -> SystemShaderComponentEntries<'_> {
        let Self {
            component_entries, ..
        } = self;
        SystemShaderComponentEntries {
            inner: component_entries.iter(),
        }
    }

    #[inline]
    pub fn additional_entries(&self) -> &[BindGroupLayoutEntry] {
        let Self {
            additional_entries, ..
        } = self;
        additional_entries
    }

    #[inline]
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        let Self {
            bind_group_layout, ..
        } = self;
        bind_group_layout
    }

    #[inline]
    pub fn pipeline_layout(&self) -> &PipelineLayout {
        let Self {
            pipeline_layout, ..
        } = self;
        pipeline_layout
    }

    #[inline]
    pub fn compute_pipeline(&self) -> &ComputePipeline {
        let Self {
            compute_pipeline, ..
        } = self;
        compute_pipeline
    }
}

#[derive(Debug, Clone)]
pub struct SystemShaderComponentEntries<'a> {
    inner: map::Iter<'a, GpuComponentId, Option<BindGroupLayoutEntry>>,
}

impl<'a> Iterator for SystemShaderComponentEntries<'a> {
    type Item = (GpuComponentId, Option<&'a BindGroupLayoutEntry>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(&id, entry)| (id, entry.as_ref()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(&id, entry)| (id, entry.as_ref()))
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(|(&id, entry)| (id, entry.as_ref()))
    }

    #[inline]
    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
    {
        let Self { inner } = self;
        inner.map(|(&id, entry)| (id, entry.as_ref())).collect()
    }
}

impl DoubleEndedIterator for SystemShaderComponentEntries<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|(&id, entry)| (id, entry.as_ref()))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(|(&id, entry)| (id, entry.as_ref()))
    }
}

impl ExactSizeIterator for SystemShaderComponentEntries<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for SystemShaderComponentEntries<'_> {}
