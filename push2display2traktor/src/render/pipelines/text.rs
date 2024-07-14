use glyphon::{
    cosmic_text::Align, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution,
    Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{Device, Extent3d, Queue, RenderPass};

use super::Pipeline;
use crate::traktor::TraktorState;

pub struct TextPipe {
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    texts: Vec<TextStorageData>,
    font_system: FontSystem,
}

impl Pipeline<TraktorState> for TextPipe {
    fn new(device: &Device, queue: &Queue, size: &Extent3d) -> Self {
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut viewport = Viewport::new(&device, &cache);
        let mut atlas =
            TextAtlas::new(&device, &queue, &cache, wgpu::TextureFormat::Rgba8UnormSrgb);
        let renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

        // This shouldn't change in our program, on normal windows applications
        // this should be called in the resize handler
        viewport.update(
            &queue,
            Resolution {
                width: size.width,
                height: size.height,
            },
        );

        let font_system = FontSystem::new();

        Self {
            swash_cache,
            viewport,
            atlas,
            renderer,
            texts: vec![],
            font_system,
        }
    }

    fn prepare(&mut self, device: &Device, queue: &Queue) {
        if self.texts.len() < 1 {
            return;
        }
        self.renderer
            .prepare(
                &device,
                &queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                self.texts.iter().map(|x| x.to_text_area()),
                &mut self.swash_cache,
            )
            .unwrap();
    }

    fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        if self.texts.len() < 1 {
            return;
        }
        self.renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

    fn render_cleanup(&mut self) {
        if self.texts.len() < 1 {
            return;
        }
        self.atlas.trim();
    }

    // Updates the texts from the traktor state
    fn update(&mut self, state: &TraktorState) {
        for (i, fx_name) in state.iter_knob_fx_names().enumerate() {
            if self.texts.len() < i + 1 {
                self.texts.push(TextStorageData::new(
                    fx_name.clone(),
                    &mut self.font_system,
                    i as u32,
                ));
            } else {
                if self.texts[i].text != *fx_name {
                    self.texts[i].text = fx_name.to_string();
                    self.texts[i].update_buffer(&mut self.font_system);
                }
            }
        }
    }
}

/// A text helper to render a text on the screen
struct TextStorageData {
    text: String,
    buffer: Buffer,
    left: f32,
    top: f32,
}

impl TextStorageData {
    fn new(text: String, mut font_system: &mut FontSystem, fx_id: u32) -> Self {
        // Define text to draw
        let mut buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(18.0, 20.0));

        buffer.set_size(&mut font_system, Some(110 as f32), Some(150 as f32));

        // Size is hardcoded to divide the display into 8 blocks and add 5px padding
        // 960/8 - 5*2 = 110
        // 160 - 5*2 = 150
        let mut s = Self {
            buffer,
            text,
            left: 5.0 + 120.0 * fx_id as f32,
            top: 5.0,
        };
        s.update_buffer(font_system);
        s
    }

    fn update_buffer(&mut self, mut font_system: &mut FontSystem) {
        self.buffer.set_text(
            &mut font_system,
            &self.text,
            Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
        );

        // Align center
        for l in self.buffer.lines.iter_mut() {
            l.set_align(Some(Align::Center));
        }

        self.buffer.shape_until_scroll(&mut font_system, false);
    }

    fn to_text_area(&self) -> TextArea {
        TextArea {
            buffer: &self.buffer,
            left: self.left,
            top: self.top,
            scale: 1.0,
            bounds: TextBounds {
                left: self.left as i32,
                top: self.top as i32,
                right: self.left as i32 + 110,
                bottom: self.top as i32 + 150,
            },
            default_color: Color::rgb(255, 255, 255),
        }
    }
}
