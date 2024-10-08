use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue};

/// A struct that handles a uniform buffer, its bind group, and the data it contains.
/// This makes it easier to create, update, and bind uniform data in a graphics pipeline.
pub struct StorageBuffer<Data: StorageData> {
    buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub data: Data,
}

/// A trait that needs to be implemented by any data type used as uniform data in `StorageBuffer`.
/// It ensures that the data can be safely transferred to the GPU.
///
/// Implementing types should also be marked with `#[repr(C)]` to ensure they have a
/// predictable memory layout that matches the C representation.
pub trait StorageData: bytemuck::Pod + bytemuck::Zeroable {
    /// Returns a default instance of the implementing type.
    fn default() -> Self;

    /// Create the binding group for the uniform data
    fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout;
}

impl<Data: StorageData> StorageBuffer<Data> {
    /// Creates a new `StorageBuffer` with the given data and bind group layout.
    ///
    /// # Arguments
    /// - `device`: The `wgpu::Device` to use for creating the buffer and bind group.
    /// - `uniform`: The initial uniform data to put in the buffer.
    /// - `bind_group_layout`: The layout describing the structure of the bind group.
    ///
    /// # Returns
    /// A new `StorageBuffer` instance.
    pub fn new(device: &Device, uniform: Data) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = uniform.create_bind_group_layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
            data: uniform,
        }
    }

    /// Updates the buffer with new uniform data.
    ///
    /// # Arguments
    /// - `queue`: The `wgpu::Queue` to use for writing the new data to the buffer.
    ///
    /// This function updates the internal uniform data and writes it to the GPU buffer.
    pub fn prepare(&mut self, queue: &Queue) {
        // TODO i dont think we need to do this every update
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }

    /// Gets a reference to the bind group for this uniform buffer.
    ///
    /// # Returns
    /// A reference to the `wgpu::BindGroup` used to bind this uniform buffer to the pipeline.
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
