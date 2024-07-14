pub mod knobs;
pub mod text;

/// A trait that defines the required methods for a rendering pipeline.
/// This more or less follows the middleware pattern
/// see https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work
pub trait Pipeline<State> {
    /// Creates a new instance of the render pipeline.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the GPU device.
    /// * `queue` - A reference to the GPU queue.
    /// * `size` - A reference to the dimensions of the rendering target.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the implementing render pipeline.
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: &wgpu::Extent3d) -> Self;

    /// Prepares the render pipeline for rendering.
    ///
    /// This method is called before the rendering starts. It can be used to set up
    /// resources or perform any necessary pre-rendering steps.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the GPU device.
    /// * `queue` - A reference to the GPU queue.
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

    /// Executes the rendering commands.
    ///
    /// This method is called during the rendering process. It should contain the commands
    /// necessary to render the scene or objects.
    ///
    /// # Arguments
    ///
    /// * `render_pass` - A mutable reference to the render pass object, which allows
    ///                   recording rendering commands.
    fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>);

    /// Cleans up after rendering.
    ///
    /// This method is called after the rendering is complete. It can be used to release
    /// resources or perform any necessary post-rendering steps.
    fn render_cleanup(&mut self);

    /// Updates the render pipeline based on the current state.
    ///
    /// This asynchronous method is called to update the texts or other dynamic elements
    /// in the render pipeline based on the current state.
    ///
    /// # Arguments
    ///
    /// * `state` - A reference to the current state.
    ///
    /// # Returns
    ///
    /// Returns a future that resolves when the update is complete.
    fn update(&mut self, state: &State);
}
