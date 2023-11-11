use crate::gpu::GpuWrapper;

pub struct UniformSpec {
    layout: wgpu::BindGroupLayout,
    element_aligned_size: usize,
}

impl UniformSpec {
    pub fn new<T>(gpu: &GpuWrapper, name: &str, visibility: wgpu::ShaderStages) -> UniformSpec {
        let uniform_alignment = gpu.device().limits().min_uniform_buffer_offset_alignment as usize;
        let aligned_size = wgpu::util::align_to(std::mem::size_of::<T>(), uniform_alignment);

        UniformSpec {
            layout: gpu.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(name),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(aligned_size as u64),
                    },
                    count: None,
                }],
            }),
            element_aligned_size: aligned_size,
        }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn offset_of(&self, index: usize) -> wgpu::DynamicOffset {
        (self.element_aligned_size * index) as u32
    }
}

pub struct UniformStorage {
    staging: Vec<u8>,
    buffer: wgpu::Buffer,
    bind_groups: Vec<wgpu::BindGroup>,
    section_offsets: Vec<usize>,
    section_element_sizes: Vec<usize>,
}

impl UniformStorage {
    pub fn new(gpu: &GpuWrapper, name: &str, specs: &[(&UniformSpec, usize, &str)]) -> UniformStorage {
        let total_buf_size = specs.iter().fold(0, |acc, (spec, num_entries, _)| acc + spec.element_aligned_size * num_entries);
        let buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some(name),
            size: total_buf_size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut section_offsets = Vec::with_capacity(specs.len());
        let mut section_element_sizes = Vec::with_capacity(specs.len());
        let mut bind_groups = Vec::with_capacity(specs.len());
        let mut running_offset = 0;

        for (spec, num_entries, name) in specs {
            section_offsets.push(running_offset);
            section_element_sizes.push(spec.element_aligned_size);
            bind_groups.push(gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &spec.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: running_offset as u64,
                        size: wgpu::BufferSize::new(spec.element_aligned_size as u64),
                    }),
                }],
                label: Some(name),
            }));

            running_offset += spec.element_aligned_size * num_entries;
        }

        UniformStorage {
            staging: vec![0u8; total_buf_size],
            buffer,
            bind_groups,
            section_offsets,
            section_element_sizes,
        }
    }

    pub fn set_element<T: bytemuck::Pod>(&mut self, spec_index: usize, element_index: usize, value: T) {
        let offset = self.section_offsets[spec_index] + self.section_element_sizes[spec_index] * element_index;
        let bytes = bytemuck::bytes_of(&value);
        self.staging[offset..offset+std::mem::size_of::<T>()].copy_from_slice(bytes);
    }

    pub fn bind_group(&self, index: usize) -> &wgpu::BindGroup {
        &self.bind_groups[index]
    }

    pub fn update(&self, gpu: &GpuWrapper) {
        gpu.queue().write_buffer(&self.buffer, 0, &self.staging);
    }
}
