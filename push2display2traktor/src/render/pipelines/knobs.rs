use super::Pipeline;
use crate::{
    render::storage_buffer::{StorageBuffer, StorageData},
    traktor::TraktorState,
};

pub struct KnobsIndicatorPipe {
    pipeline: wgpu::RenderPipeline,
    buffer: StorageBuffer<KnobStorageData>,
}

impl Pipeline<TraktorState> for KnobsIndicatorPipe {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, _size: &wgpu::Extent3d) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("knobs.wgsl").into()),
        });

        // Create uniform buffer(s) here only for the knobs, see uniform_buffer.rs
        let buffer = StorageBuffer::new(&device, KnobStorageData::default());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Knobs indicator Pipeline"),
                bind_group_layouts: &[&buffer.bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self { pipeline, buffer }
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        self.buffer.prepare(&queue);
    }

    fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        // Draw knobs shader
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.buffer.bind_group, &[]);

        // We draw 8 knobs only as the display is limited to two effect units
        render_pass.draw(0..6, 0..8);
    }

    fn render_cleanup(&mut self) {
        return;
    }

    fn update(&mut self, state: &TraktorState) {
        self.buffer.data = KnobStorageData::from(state);
    }
}

/* -------------------------------------------------------------------------- */
/*                             My storage data                                */
/* -------------------------------------------------------------------------- */

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct KnobStorageData {
    n_knobs: u32,     // 4 bytes
    knobs: [f32; 16], // 16*4 = 64
}
// Implement the UniformData trait for KnobsStateUniformData
impl StorageData for KnobStorageData {
    /// Returns a default instance of the implementing type.
    fn default() -> Self {
        Self {
            n_knobs: 4,
            knobs: [1.0; 16],
        }
    }

    fn create_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        return device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Knobs bind group layout"),
        });
    }
}

impl KnobStorageData {
    pub fn new(n_knobs: u32, knobs: [f32; 16]) -> Self {
        Self { n_knobs, knobs }
    }
}

impl From<&TraktorState> for KnobStorageData {
    fn from(state: &TraktorState) -> Self {
        let n_knobs = 8;

        let mut knobs: [f32; 16] = [0.0; 16]; // Initialize an array of 16 zeros
        for (i, v) in state.iter_knob_values().enumerate() {
            knobs[i] = *v as f32;
        }
        return Self { n_knobs, knobs };
    }
}
