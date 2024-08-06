#![feature(portable_simd)]
mod push2;
mod render;
mod traktor;

use crate::push2::Push2Display;
use crate::render::render_loop;
use crate::traktor::TraktorState;

use std::sync::Arc;
use tokio::sync::Mutex;
use traktor::create_server;


#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {

    // Create state that hold traktor data arc mutex to share between tokio threads
    let state = Arc::new(Mutex::new(TraktorState::default()));

    // start tasks in threads
    let h1 = start_render_task(&state);

    h1.await;

    // Keep the main task alive indefinitely
    start_traktor_handler(state).await
}

async fn start_render_task(state: &Arc<Mutex<TraktorState>>) {
    let s = Arc::clone(state);

    // Create display driver
    let display = Push2Display::new().unwrap();

    // create graphics pipeline for display
    let graphics =
        render::graphics::Graphics::new(push2::DISPLAY_WIDTH as u32, push2::DISPLAY_HEIGHT as u32)
            .await;

    tokio::spawn(async move { render_loop(graphics, display, s).await });
}

async fn start_traktor_handler(state: Arc<Mutex<TraktorState>>) -> std::io::Result<()> {
    create_server(state).await
}
