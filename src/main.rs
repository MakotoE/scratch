#[macro_use]
extern crate conrod_core;

use conrod_core::text::GlyphCache;
use conrod_core::widget::Canvas;
use conrod_core::{Borderable, Theme, Widget};
use piston_window::texture::UpdateTexture;
use piston_window::{
    G2d, G2dTexture, OpenGL, PistonWindow, Size, TextureSettings, UpdateEvent, Window,
    WindowSettings,
};

widget_ids! {
    pub struct Ids {
        page,
    }
}

fn main() {
    const PAGE_SIZE: Size = Size {
        width: 480.0,
        height: 500.0,
    };

    let mut window: PistonWindow = WindowSettings::new("Scratch", PAGE_SIZE)
        .graphics_api(OpenGL::V3_2)
        .samples(8)
        .vsync(true)
        .build()
        .unwrap();

    let mut ui = conrod_core::UiBuilder::new([PAGE_SIZE.width, PAGE_SIZE.height])
        .theme(Theme::default())
        .build();

    ui.fonts
        .insert_from_file("assets/Roboto-Regular.ttf")
        .unwrap();

    let mut texture_context = window.create_texture_context();

    let mut text_texture_cache = G2dTexture::from_memory_alpha(
        &mut texture_context,
        &[128; (PAGE_SIZE.width * PAGE_SIZE.height) as usize],
        PAGE_SIZE.width as u32,
        PAGE_SIZE.height as u32,
        &TextureSettings::new(),
    )
    .unwrap();

    let mut glyph_cache = GlyphCache::builder()
        .dimensions(PAGE_SIZE.width as u32, PAGE_SIZE.height as u32)
        .scale_tolerance(0.1)
        .position_tolerance(0.1)
        .build();

    let image_map = conrod_core::image::Map::new();

    let mut text_vertex_data: Vec<u8> = Vec::new();

    let ids = Ids::new(ui.widget_id_generator());

    while let Some(event) = window.next() {
        let size = window.size();
        if let Some(e) = conrod_piston::event::convert(event.clone(), size.width, size.height) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui_cell = ui.set_widgets();
            Canvas::new().pad(10.0).set(ids.page, &mut ui_cell);
        });

        let cache_queued_glyphs = |_: &mut G2d,
                                   cache: &mut G2dTexture,
                                   rect: conrod_core::text::rt::Rect<u32>,
                                   data: &[u8]| {
            let offset = [rect.min.x, rect.min.y];
            let size = [rect.width(), rect.height()];
            let format = piston_window::texture::Format::Rgba8;
            text_vertex_data.clear();
            text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
            UpdateTexture::update(
                cache,
                &mut texture_context,
                format,
                &text_vertex_data[..],
                offset,
                size,
            )
            .unwrap()
        };

        window.draw_2d(&event, |context, graphics, _device| {
            if let Some(primitives) = ui.draw_if_changed() {
                conrod_piston::draw::primitives(
                    primitives,
                    context,
                    graphics,
                    &mut text_texture_cache,
                    &mut glyph_cache,
                    &image_map,
                    cache_queued_glyphs,
                    |img| img,
                );
            }
        });
    }
}
