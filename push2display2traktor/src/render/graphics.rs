use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};
use wgpu::{Adapter, Buffer, Device, Extent3d, Instance, Queue, Texture, TextureView};

use crate::traktor::TraktorState;

use super::pipelines::{knobs::KnobsIndicatorPipe, text::TextPipe, Pipeline};

pub struct Graphics {
    instance: Instance,
    adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    render_target: Texture,
    texture_view: TextureView,
    output_staging_buffer: Buffer,
    pub size: Extent3d,

    // Current knob state
    knobs_pipe: KnobsIndicatorPipe,
    // Text render system for the effect names
    text_pipe: TextPipe,
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

        //-----------------------------------------------
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

        let texture_view = render_target.create_view(&Default::default());

        //-----------------------------------------------
        // Pipelines
        let knobs_pipe = KnobsIndicatorPipe::new(&device, &queue, &size);
        let text_pipe = TextPipe::new(&device, &queue, &size);
        //-----------------------------------------------

        Self {
            instance,
            adapter,
            device,
            queue,
            texture_view,
            render_target,
            output_staging_buffer,
            size,
            knobs_pipe,
            text_pipe,
        }
    }

    // Writes the current frame to the buffer
    pub async fn render(&mut self) -> Vec<u8> {
        {
            // Prepare all pipes
            self.knobs_pipe.prepare(&self.device, &self.queue);
            self.text_pipe.prepare(&self.device, &self.queue);

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
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                // Draw text
                self.knobs_pipe.render(&mut render_pass);
                self.text_pipe.render(&mut render_pass);
            }

            // The texture now contains our rendered image
            // which we can than pass to a buffer and than to the display
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

            // Cleanup pipelines
            self.text_pipe.render_cleanup();
        }

        // Wait for bufferslice
        let buffer_slice = self.output_staging_buffer.slice(..);
        let (tx, rx) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.await.unwrap().unwrap();

        // Copy texture to a buffer and return
        let data = buffer_slice.get_mapped_range();
        let dat = data.to_vec();
        drop(data);
        self.output_staging_buffer.unmap();
        return dat;
    }

    pub async fn update(&mut self, state: &Arc<Mutex<TraktorState>>) {
        let s = state.lock().await;

        // Update pipelines
        self.knobs_pipe.update(&s);
        self.text_pipe.update(&s);
    }
}
