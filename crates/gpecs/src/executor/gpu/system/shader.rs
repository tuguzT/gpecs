use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    num::NonZeroU32,
};

use indexmap::map::Iter as IndexMapIter;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, BufferSize, ComputePipeline, ComputePipelineDescriptor, Device, Label,
    PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, ShaderModule,
    ShaderStages,
};

use crate::{
    archetype::{collect::try_collect_opt_components, error::ArchetypeError},
    component::registry::{ComponentInfo, ComponentRegistry, ErasedDropComponentDescriptor},
    entity::Entity,
    executor::gpu::component::registry::GpuComponentId,
    hash::IndexMap,
};

use super::registry::{GpuComponentAccess, GpuSystemDescriptor, GpuSystemId};

#[derive(Debug)]
pub struct GpuSystemShader {
    label: Option<String>,
    entity_entry: Option<GpuSystemShaderEntry>,
    component_entries: IndexMap<GpuComponentId, Option<GpuSystemShaderEntry>>,
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
    ) -> Result<Self, ArchetypeError>
    where
        C: IntoIterator<Item = (GpuComponentId, GpuComponentAccess)>,
        B: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        const ENTITY_MIN_BINDING_SIZE: BufferSize =
            BufferSize::new(size_of::<Entity>() as u64).expect("`Entity` cannot be ZST");

        let GpuSystemDescriptor {
            label,
            shader_module,
            entry_point,
            workgroup_size,
            bind_entities,
            bind_components,
            additional_bindings,
        } = descriptor;

        let entity_entry = bind_entities.then_some(GpuSystemShaderEntry {
            binding_index: 0,
            binding_access: GpuComponentAccess::ReadOnly,
            min_binding_size: ENTITY_MIN_BINDING_SIZE,
        });

        let component_ids = try_collect_opt_components(
            bind_components.into_iter().map(|(id, access)| {
                let info = components.get_component_info(id.into())?;
                Some((id, info, access))
            }),
            |map, (id, info, access)| IndexMap::insert(map, id, (info, access)).is_none(),
            |&(component_id, _, _)| component_id.into(),
        )?;
        let component_entry =
            |index: usize, info: &ComponentInfo<ErasedDropComponentDescriptor>, binding_access| {
                let size_of_component = info.as_meta().descriptor().layout().size();
                let size_of_component = size_of_component
                    .try_into()
                    .expect("size of component should fit in `u64`");
                let min_binding_size = BufferSize::new(size_of_component)?;

                let binding_index = (index + usize::from(bind_entities))
                    .try_into()
                    .expect("count of bindings should fit in `u32`");
                let component_entry = GpuSystemShaderEntry {
                    binding_index,
                    binding_access,
                    min_binding_size,
                };
                Some(component_entry)
            };
        let component_entries: IndexMap<_, _> = component_ids
            .into_iter()
            .enumerate()
            .map(|(index, (component_id, (info, access)))| {
                (component_id, component_entry(index, info, access))
            })
            .collect();

        let additional_entries: Box<_> = additional_bindings.into_iter().collect();

        let bind_group_layout_label = match label {
            Some(label) => format!("`gpecs` {system_id:#} [{label}] bind group layout"),
            None => format!("`gpecs` {system_id:#} bind group layout"),
        };
        let bind_group_layout_entries = entity_entry
            .into_iter()
            .chain(component_entries.values().copied().flatten())
            .map(BindGroupLayoutEntry::from)
            .chain(additional_entries.iter().copied())
            .collect::<Box<_>>();
        let bind_group_layout_desc = BindGroupLayoutDescriptor {
            label: Some(&bind_group_layout_label),
            entries: bind_group_layout_entries.as_ref(),
        };
        let bind_group_layout = gpu_device.create_bind_group_layout(&bind_group_layout_desc);

        let pipeline_layout_label = match label {
            Some(label) => format!("`gpecs` {system_id:#} [{label}] pipeline layout"),
            None => format!("`gpecs` {system_id:#} pipeline layout"),
        };
        let pipeline_layout_desc = PipelineLayoutDescriptor {
            label: Some(&pipeline_layout_label),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        };
        let pipeline_layout = gpu_device.create_pipeline_layout(&pipeline_layout_desc);

        let compute_pipeline_label = match label {
            Some(label) => format!("`gpecs` {system_id:#} [{label}] compute pipeline"),
            None => format!("`gpecs` {system_id:#} compute pipeline"),
        };
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
            label: label.map(ToOwned::to_owned),
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
    pub fn label(&self) -> Label<'_> {
        let Self { label, .. } = self;
        label.as_deref()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct GpuSystemShaderEntry {
    pub binding_index: u32,
    pub binding_access: GpuComponentAccess,
    pub min_binding_size: BufferSize,
}

impl From<GpuSystemShaderEntry> for BindGroupLayoutEntry {
    #[inline]
    fn from(value: GpuSystemShaderEntry) -> Self {
        let GpuSystemShaderEntry {
            binding_index,
            binding_access,
            min_binding_size,
        } = value;

        let read_only = match binding_access {
            GpuComponentAccess::ReadOnly => true,
            GpuComponentAccess::ReadWrite => false,
        };
        BindGroupLayoutEntry {
            binding: binding_index,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only },
                min_binding_size: Some(min_binding_size),
                has_dynamic_offset: false,
            },
            count: None,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GpuSystemShaderEntries<'a> {
    pub entities: Option<&'a GpuSystemShaderEntry>,
    pub components: GpuSystemShaderComponentEntries<'a>,
    pub additional: &'a [BindGroupLayoutEntry],
}

#[derive(Clone)]
pub struct GpuSystemShaderComponentEntries<'a> {
    inner: IndexMapIter<'a, GpuComponentId, Option<GpuSystemShaderEntry>>,
}

impl Debug for GpuSystemShaderComponentEntries<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl<'a> Iterator for GpuSystemShaderComponentEntries<'a> {
    type Item = (GpuComponentId, Option<&'a GpuSystemShaderEntry>);

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
