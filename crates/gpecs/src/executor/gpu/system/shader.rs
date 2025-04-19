use std::{iter::FusedIterator, num::NonZeroU64};

use indexmap::{IndexMap, IndexSet};
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

use super::registry::GpuSystemId;

#[derive(Debug)]
pub struct SystemShader {
    entities_bind_group_layout_entry: Option<BindGroupLayoutEntry>,
    components_bind_group_layout_entries: IndexMap<GpuComponentId, Option<BindGroupLayoutEntry>>,
    shader_module: ShaderModule,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    compute_pipeline: ComputePipeline,
}

impl SystemShader {
    #[inline]
    #[allow(unsafe_code)]
    pub(super) fn new<I>(
        components: &ComponentRegistry,
        gpu_device: &Device,
        shader_module: ShaderModule,
        entry_point: Option<&str>,
        system_id: GpuSystemId,
        bind_entities: bool,
        component_ids: I,
    ) -> Result<Self, DuplicateComponentError>
    where
        I: IntoIterator<Item = GpuComponentId>,
    {
        let component_ids = component_ids.into_iter().map(Into::into);
        let component_ids = try_collect_component_ids(component_ids, IndexSet::<_>::insert)?;
        let component_ids: IndexSet<_> = component_ids
            .into_iter()
            .map(|id| unsafe { GpuComponentId::from_id(id) })
            .collect();

        let max_bind_group_layout_entries = component_ids.len() + (bind_entities as usize);
        let mut bind_group_layout_entries = Vec::with_capacity(max_bind_group_layout_entries);

        let mut entities_bind_group_layout_entry = None;
        if bind_entities {
            entities_bind_group_layout_entry = BindGroupLayoutEntry {
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
            }
            .into();
            bind_group_layout_entries.push(entities_bind_group_layout_entry.unwrap());
        }

        let mut components_bind_group_layout_entries =
            IndexMap::with_capacity(max_bind_group_layout_entries);
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
                components_bind_group_layout_entries.insert(component_id, None);
                continue;
            };

            let component_bind_group_layout_entry = BindGroupLayoutEntry {
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
            components_bind_group_layout_entries.insert(
                component_id,
                Some(component_bind_group_layout_entry.clone()),
            );
            bind_group_layout_entries.push(component_bind_group_layout_entry);
        }

        let bind_group_layout_label = format!("`gpecs` {system_id:?} bind group layout");
        let bind_group_layout_desc = BindGroupLayoutDescriptor {
            label: Some(&bind_group_layout_label),
            entries: &bind_group_layout_entries,
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
            entities_bind_group_layout_entry,
            components_bind_group_layout_entries,
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
    pub fn entities_bind_group_layout_entry(&self) -> Option<&BindGroupLayoutEntry> {
        let Self {
            entities_bind_group_layout_entry,
            ..
        } = self;
        entities_bind_group_layout_entry.as_ref()
    }

    #[inline]
    // TODO: add specific iterator type
    pub fn components_bind_group_layout_entries(
        &self,
    ) -> impl DoubleEndedIterator<Item = (GpuComponentId, Option<&BindGroupLayoutEntry>)>
           + ExactSizeIterator
           + FusedIterator
           + Clone {
        let Self {
            components_bind_group_layout_entries,
            ..
        } = self;
        components_bind_group_layout_entries
            .iter()
            .map(|(&id, entry)| (id, entry.as_ref()))
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
