mod fps_controller;
pub mod graphics;
pub mod storage_buffer;

use crate::push2::{encode_buffer, rgba8_to_bgr565, Push2Display};
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

        // Render to array
        let rgba_data = graphics.render().await;
        // Convert RGBA8 to bgr565 and send to push
        let bgr565_data = rgba8_to_bgr565(&rgba_data);
        let p = encode_buffer(&bgr565_data);
        display.send_buffer(&p).unwrap();

        fps_controller.end_frame().await;
    }
}
