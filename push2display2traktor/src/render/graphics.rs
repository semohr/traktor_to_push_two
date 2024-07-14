use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};
use wgpu::{
    Adapter, Buffer, Device, Extent3d, Instance, Queue, RenderPipeline, Texture, TextureView,
};

use crate::traktor::TraktorState;

use super::storage_buffer::{StorageBuffer, StorageData, TraktorStateStorageData};

pub struct Graphics {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    render_target: Texture,
    texture_view: TextureView,
    output_staging_buffer: Buffer,
    pipeline: RenderPipeline,
    pub uniform_buffer: StorageBuffer<TraktorStateStorageData>,
    pub size: Extent3d,
}

impl Graphics {
    pub async fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // We use the output texture to map the output to a buffer see also
        // https://sotrh.github.io/learn-wgpu/showcase/windowless/#a-triangle-without-a-window
        let render_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size * size.width * size.height) as wgpu::BufferAddress;
        let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create uniform buffer(s) here only for the knobs, see uniform_buffer.rs
        let uniform_buffer = StorageBuffer::new(&device, TraktorStateStorageData::default());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_buffer.bind_group_layout],
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

        //-----------------------------------------------

        let texture_view = render_target.create_view(&Default::default());

        Self {
            instance,
            adapter,
            device,
            queue,
            texture_view,
            render_target,
            output_staging_buffer,
            pipeline,
            size,
            uniform_buffer,
        }
    }

    // Writes the current frame to the buffer
    pub async fn render(&self) -> Vec<u8> {
        {
            let mut command_encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut render_pass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &self.texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.uniform_buffer.bind_group, &[]);
                render_pass.draw(0..6, 0..8);
            }

            // The texture now contains our rendered image
            let u32_size = std::mem::size_of::<u32>() as u32;
            command_encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &self.render_target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &self.output_staging_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((self.size.width * u32_size) as u32),
                        rows_per_image: Some(self.size.height as u32),
                    },
                },
                self.size,
            );
            self.queue.submit(Some(command_encoder.finish()));
        }

        /* -------------------------------------------------------------------------- */
        /*                               copy to buffer                               */
        /* -------------------------------------------------------------------------- */

        let buffer_slice = self.output_staging_buffer.slice(..);

        let (tx, rx) = oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let dat = data.to_vec();
        drop(data);
        self.output_staging_buffer.unmap();
        return dat;
    }

    pub async fn update(&mut self, state: &Arc<Mutex<TraktorState>>) {
        let s = state.lock().await;
        self.uniform_buffer.data = s.to_uniform();
        self.uniform_buffer.update(&self.queue);
    }
}
