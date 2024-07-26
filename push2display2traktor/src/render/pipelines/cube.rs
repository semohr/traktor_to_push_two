use std::time::Instant;


use super::Pipeline;
use crate::{
    render::storage_buffer::{StorageBuffer, StorageData},
    traktor::TraktorState,
};

pub struct CubePipeline {
    pipeline: wgpu::RenderPipeline,
    buffer: StorageBuffer<TimeStorageData>,
    last_call: Instant,
}

impl Pipeline<TraktorState> for CubePipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, _size: &wgpu::Extent3d) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Cube"),
            source: wgpu::ShaderSource::Wgsl(include_str!("cube.wgsl").into()),
        });

        let buffer = StorageBuffer::new(&device, TimeStorageData::default());
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cube Pipeline Layout"),
                bind_group_layouts: &[&buffer.bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cube Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[], // Define the vertex buffer layout here
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

        Self {
            pipeline,
            buffer,
            last_call: Instant::now(),
        }
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        let duration = self.last_call.elapsed();
        self.buffer.data.time = duration.as_secs_f32();
        self.buffer.prepare(queue);
    }

    fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.buffer.bind_group, &[]);

        // Assuming you have set up your vertex buffer elsewhere
        render_pass.draw(0..3, 0..1);
    }

    fn render_cleanup(&mut self) {}

    fn update(&mut self, state: &TraktorState) {
        let _ = state;
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeStorageData {
    time: f32,
}

impl StorageData for TimeStorageData {
    fn default() -> Self {
        Self { time: 0.0 }
    }

    fn create_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Time bind group layout"),
        })
    }
}

