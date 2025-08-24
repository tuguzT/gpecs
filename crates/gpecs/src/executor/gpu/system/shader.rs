use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    num::{NonZeroU32, NonZeroU64},
};

use indexmap::map::Iter as IndexMapIter;
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
    hash::{IndexMap, IndexSet},
};

use super::registry::{GpuSystemDescriptor, GpuSystemId};

#[derive(Debug)]
pub struct GpuSystemShader {
    entity_entry: Option<BindGroupLayoutEntry>,
    component_entries: IndexMap<GpuComponentId, Option<BindGroupLayoutEntry>>,
    additional_entries: Box<[BindGroupLayoutEntry]>,
    shader_module: ShaderModule,
    workgroup_size: Option<NonZeroU32>,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    compute_pipeline: ComputePipeline,
}

impl GpuSystemShader {
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
            workgroup_size,
            bind_entities,
            bind_components,
            additional_bindings,
        } = descriptor;

        let entity_entry = bind_entities.then_some(buffer_entry(0, ENTITY_MIN_BINDING_SIZE));

        let component_ids = try_collect_component_ids(bind_components, IndexSet::<_>::insert)?;
        let component_entry = |index: usize, component_id: GpuComponentId| {
            let Some(info) = components.get_component_info(component_id.into()) else {
                unreachable!("component {component_id:?} should exist");
            };

            let size_of_component = info.descriptor().layout().size();
            let size_of_component = size_of_component
                .try_into()
                .expect("size of component should fit in `u64`");
            let min_binding_size = NonZeroU64::new(size_of_component)?;

            let binding = (index + usize::from(bind_entities))
                .try_into()
                .expect("count of bindings should fit in `u32`");
            let component_entry = buffer_entry(binding, min_binding_size);
            Some(component_entry)
        };
        let component_entries: IndexMap<_, _> = component_ids
            .into_iter()
            .enumerate()
            .map(|(index, component_id)| (component_id, component_entry(index, component_id)))
            .collect();

        let additional_entries: Box<_> = additional_bindings.into_iter().collect();

        let bind_group_layout_label = format!("`gpecs` {system_id:?} bind group layout");
        let bind_group_layout_entries = entity_entry
            .into_iter()
            .chain(component_entries.values().copied().flatten())
            .chain(additional_entries.iter().copied())
            .collect::<Box<_>>();
        let bind_group_layout_desc = BindGroupLayoutDescriptor {
            label: Some(&bind_group_layout_label),
            entries: bind_group_layout_entries.as_ref(),
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
            shader_module,
            workgroup_size,
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
    pub fn workgroup_size(&self) -> Option<NonZeroU32> {
        let Self { workgroup_size, .. } = *self;
        workgroup_size
    }

    #[inline]
    pub fn bind_group_layout_entries(&self) -> GpuSystemShaderEntries<'_> {
        let Self {
            entity_entry,
            component_entries,
            additional_entries,
            ..
        } = self;

        GpuSystemShaderEntries {
            entities: entity_entry.as_ref(),
            components: GpuSystemShaderComponentEntries {
                inner: component_entries.iter(),
            },
            additional: additional_entries.as_ref(),
        }
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
pub struct GpuSystemShaderEntries<'a> {
    pub entities: Option<&'a BindGroupLayoutEntry>,
    pub components: GpuSystemShaderComponentEntries<'a>,
    pub additional: &'a [BindGroupLayoutEntry],
}

#[derive(Clone)]
pub struct GpuSystemShaderComponentEntries<'a> {
    inner: IndexMapIter<'a, GpuComponentId, Option<BindGroupLayoutEntry>>,
}

impl Debug for GpuSystemShaderComponentEntries<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl<'a> Iterator for GpuSystemShaderComponentEntries<'a> {
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

impl DoubleEndedIterator for GpuSystemShaderComponentEntries<'_> {
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

impl ExactSizeIterator for GpuSystemShaderComponentEntries<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuSystemShaderComponentEntries<'_> {}

const ENTITY_MIN_BINDING_SIZE: NonZeroU64 =
    NonZeroU64::new(size_of::<Entity>() as u64).expect("size of `Entity` cannot be zero");

#[inline]
fn buffer_entry(binding: u32, min_binding_size: NonZeroU64) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            min_binding_size: Some(min_binding_size),
            has_dynamic_offset: false,
        },
        count: None,
    }
}
