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
    knob_texts: Vec<TextStorageData>,
    other_texts: Vec<TextStorageData>,
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

        let mut font_system = FontSystem::new();
        let other_texts = vec![
            TextStorageData::new(
                "LOAD A".to_string(),
                &mut font_system,
                5.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "SYNC A".to_string(),
                &mut font_system,
                5.0 + 120.0 * 1.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "FX 1".to_string(),
                &mut font_system,
                5.0 + 120.0 * 2.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "FX 2".to_string(),
                &mut font_system,
                5.0 + 120.0 * 3.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "FX 1".to_string(),
                &mut font_system,
                5.0 + 120.0 * 4.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "FX 2".to_string(),
                &mut font_system,
                5.0 + 120.0 * 5.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "SYNC B".to_string(),
                &mut font_system,
                5.0 + 120.0 * 6.0,
                160.0 - 15.0 - 15.0,
            ),
            TextStorageData::new(
                "LOAD B".to_string(),
                &mut font_system,
                5.0 + 120.0 * 7.0,
                160.0 - 15.0 - 18.0,
            ),
        ];

        Self {
            swash_cache,
            viewport,
            atlas,
            renderer,
            knob_texts: vec![],
            other_texts,
            font_system,
        }
    }

    fn prepare(&mut self, device: &Device, queue: &Queue) {
        if self.knob_texts.len() < 1 {
            return;
        }

        // Combine vectors
        let knobs = self.knob_texts.iter().map(|x| x.to_text_area());
        let other = self.other_texts.iter().map(|x| x.to_text_area());
        let text = knobs.chain(other);

        self.renderer
            .prepare(
                &device,
                &queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text.collect::<Vec<TextArea>>(),
                &mut self.swash_cache,
            )
            .unwrap();
    }

    fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        if self.knob_texts.len() < 1 {
            return;
        }
        self.renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

    fn render_cleanup(&mut self) {
        if self.knob_texts.len() < 1 {
            return;
        }
        self.atlas.trim();
    }

    // Updates the texts from the traktor state
    fn update(&mut self, state: &TraktorState) {
        // Update the 16 knob texts
        for (i, fx_name) in state.iter_knob_fx_names().enumerate() {
            if self.knob_texts.len() < i + 1 {
                self.knob_texts.push(TextStorageData::new_knob(
                    fx_name.clone(),
                    &mut self.font_system,
                    i as u32,
                ));
            } else {
                if self.knob_texts[i].text != *fx_name {
                    self.knob_texts[i].text = fx_name.to_string();
                    self.knob_texts[i].update_buffer(&mut self.font_system);
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
    fn new_knob(text: String, font_system: &mut FontSystem, fx_id: u32) -> Self {
        // Size is hardcoded to divide the display into 8 blocks and add 5px padding
        // 960/8 - 5*2 = 110
        // 160 - 10*2 = 140
        return Self::new(text, font_system, 5.0 + 120.0 * fx_id as f32, 15.0);
    }

    fn new(text: String, mut font_system: &mut FontSystem, left: f32, top: f32) -> Self {
        // Define text to draw
        let mut buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(18.0, 20.0));
        buffer.set_size(&mut font_system, Some(110 as f32), Some(150 as f32));

        let mut s = Self {
            buffer,
            text,
            left,
            top,
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
