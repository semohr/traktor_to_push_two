use std::collections::VecDeque;
use std::time::{Duration, Instant};

use tokio::time::sleep;

/// FPSController manages frame timing and FPS calculation.
pub struct FPSController {
    target_duration: Duration,
    // Keep times for fps calculation i.e. frame starts and frame ends
    frame_start: VecDeque<Instant>,
    frame_end: VecDeque<Instant>,

    #[cfg(feature = "verbose")]
    last_print_time: Instant,
}

impl Default for FPSController {
    /// Constructs a new FPSController with default settings.
    fn default() -> Self {
        Self {
            frame_start: VecDeque::with_capacity(128),
            frame_end: VecDeque::with_capacity(128),
            target_duration: Duration::from_secs_f64(1. / 60.),
            #[cfg(feature = "verbose")]
            last_print_time: Instant::now(),
        }
    }
}

impl FPSController {
    /// Records the start of a new frame.
    pub fn start_frame(&mut self) {
        self.frame_start.push_back(Instant::now());
    }

    /// Records the end of the current frame and manages FPS and timing.
    pub async fn end_frame(&mut self) {
        let now = Instant::now();
        self.frame_end.push_back(now);

        // Ensure frame start and end queues have the same length
        while self.frame_end.len() > self.frame_start.len() {
            self.frame_end.pop_front();
        }

        // Conditional compilation for printing FPS
        #[cfg(feature = "verbose")]
        {
            if now.duration_since(self.last_print_time) >= Duration::from_secs(2) {
                self.print_current_fps();
                self.last_print_time = now;
            }
        }

        // Sleep to maintain target FPS
        let elapsed = match self.frame_start.back() {
            Some(time) => time.elapsed(),
            None => Duration::from_secs(0),
        };
        if elapsed < self.target_duration {
            sleep(self.target_duration - elapsed).await;
        }
    }

    /// Prints current FPS and average frame time if verbose feature is enabled.
    #[cfg(feature = "verbose")]
    fn print_current_fps(&self) {
        println!("Current FPS: {:.2}", self.fps());
        println!(
            "Average Frame Time: {:.2}ms",
            self.avg_frame_time() * 1000.0
        );
    }

    /// Calculates the average frame time in seconds.
    fn avg_frame_time(&self) -> f64 {
        if self.frame_start.is_empty() {
            return 0.0;
        }

        let mut total_duration = Duration::from_secs(0);
        let mut frame_count = 0;

        for i in 0..self.frame_end.len() {
            if let Some(start_time) = self.frame_start.get(i) {
                if let Some(end_time) = self.frame_end.get(i) {
                    total_duration += end_time.duration_since(*start_time);
                    frame_count += 1;
                }
            }
        }

        if frame_count > 0 {
            let avg_duration = total_duration / frame_count as u32;
            avg_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Calculates the current FPS.
    fn fps(&self) -> f64 {
        // Count frames within the last second
        let now = Instant::now();
        let one_second_ago = now - Duration::from_secs(1);

        let mut count = 1;
        for &start_time in self.frame_start.iter().rev() {
            if start_time < one_second_ago {
                break;
            }
            count += 1;
        }

        if count > 0 {
            let fps = count as f64;
            fps
        } else {
            0.0 // No frames recorded in the last second
        }
    }
}
