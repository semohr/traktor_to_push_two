mod fps_controller;
pub mod graphics;
mod pipelines;
pub mod storage_buffer;
use crate::push2::Push2Display;
use crate::traktor::TraktorState;

use fps_controller::FPSController;
use graphics::Graphics;

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn render_loop(
    mut graphics: Graphics,
    display: Push2Display,
    state: Arc<Mutex<TraktorState>>,
) {
    let mut fps_controller = FPSController::default();

    loop {
        fps_controller.start_frame();

        // Update buffers via state
        graphics.update(&state).await;

        // Render to push display
        let rgba_data = graphics.render().await;
        display.send_rgba8(&rgba_data).unwrap();

        fps_controller.end_frame().await;
    }
}
