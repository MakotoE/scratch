#[macro_use]
extern crate conrod_core;

use anyhow::Result;
use conrod_core::text::GlyphCache;
use conrod_core::widget::{Button, Canvas, FileNavigator};
use conrod_core::{Borderable, Color, Colorable, Positionable, Sizeable, Theme, Widget};
use graphics::math::Matrix2d;
use graphics::rectangle::Shape;
use graphics::{DrawState, Rectangle};
use piston_window::texture::UpdateTexture;
use piston_window::{
    G2d, G2dTexture, G2dTextureContext, OpenGL, PistonWindow, Size, Texture, TextureSettings,
    UpdateEvent, Window, WindowSettings,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptionError {
    #[error("option is None")]
    Option,
}

widget_ids! {
    pub struct Ids {
        navigator,
        green_flag_button,
        stop_button,
    }
}

fn main() -> Result<()> {
    const PAGE_SIZE: Size = Size {
        width: 520.0,
        height: 520.0,
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

    let mut image_map = conrod_core::image::Map::new();
    let green_flag_id = image_map.insert(image_texture(
        &mut texture_context,
        Path::new("assets/green_flag.svg"),
    )?);
    let stop_id = image_map.insert(image_texture(
        &mut texture_context,
        Path::new("assets/stop.svg"),
    )?);

    let mut character_cache = window.load_font("assets/Roboto-Regular.ttf").unwrap();

    let mut text_vertex_data: Vec<u8> = Vec::new();

    let ids = Ids::new(ui.widget_id_generator());

    let mut selected_path: Option<PathBuf> = None;

    while let Some(event) = window.next() {
        let size = window.size();
        if let Some(e) = conrod_piston::event::convert(event.clone(), size.width, size.height) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui_cell = ui.set_widgets();

            Button::image(green_flag_id)
                .top_left_with_margins(10.0, 25.0)
                .w_h(30.0, 30.0)
                .set(ids.green_flag_button, &mut ui_cell);

            Button::image(stop_id)
                .top_left_with_margins(10.0, 70.0)
                .w_h(30.0, 30.0)
                .set(ids.stop_button, &mut ui_cell);

            if let Some(path) = &selected_path {
            } else {
                // let navigator = FileNavigator::all(&Path::new("."))
                //     .x(0.0)
                //     .y(0.0)
                //     .set(ids.navigator, &mut ui_cell);
                // for event in navigator {
                //     if let conrod_core::widget::file_navigator::Event::ChangeSelection(
                //         mut path_vec,
                //     ) = event
                //     {
                //         for path in path_vec.drain(..) {
                //             if !path.is_dir() {
                //                 selected_path = Some(path);
                //                 break;
                //             }
                //         }
                //     }
                // }
            }
        });

        window.draw_2d(&event, |context, graphics, device| {
            if let Some(primitives) = ui.draw_if_changed() {
                draw_border(&context.draw_state, context.transform, graphics);

                let cache_queued_glyphs = |_: &mut G2d,
                                           cache: &mut G2dTexture,
                                           rect: conrod_core::text::rt::Rect<u32>,
                                           data: &[u8]| {
                    text_vertex_data.clear();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(
                        cache,
                        &mut texture_context,
                        piston_window::texture::Format::Rgba8,
                        &text_vertex_data[..],
                        [rect.min.x, rect.min.y],
                        [rect.width(), rect.height()],
                    )
                    .unwrap()
                };

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

                texture_context.encoder.flush(device);
                character_cache.factory.encoder.flush(device);
            }
        });
    }

    Ok(())
}

fn draw_border(draw_state: &DrawState, transform: Matrix2d, graphics: &mut G2d) {
    let rectangle = Rectangle {
        color: [1.0, 1.0, 1.0, 1.0],
        shape: Shape::Square,
        border: None,
    };
    // Top
    rectangle.draw([0.0, 0.0, 520.0, 50.0], draw_state, transform, graphics);
    // Bottom
    rectangle.draw([0.0, 410.0, 520.0, 520.0], draw_state, transform, graphics);
    // Left
    rectangle.draw([0.0, 50.0, 20.0, 410.0], draw_state, transform, graphics);
    // Right
    rectangle.draw([500.0, 50.0, 520.0, 410.0], draw_state, transform, graphics);
}

fn image_texture(
    texture_context: &mut G2dTextureContext,
    path: &Path,
) -> Result<Texture<gfx_device_gl::Resources>> {
    let mut options = usvg::Options::default();
    options.fontdb.load_system_fonts();
    let tree = usvg::Tree::from_file(path, &options)?;
    let size = tree.svg_node().size.to_screen_size();
    let mut pixmap =
        tiny_skia::Pixmap::new(size.width() * 2, size.height() * 2).ok_or(OptionError::Option)?;
    let width = pixmap.width();
    let height = pixmap.height();

    resvg::render(&tree, usvg::FitTo::Zoom(2.0), pixmap.as_mut()).ok_or(OptionError::Option)?;
    let image =
        image::ImageBuffer::from_raw(width, height, pixmap.take()).ok_or(OptionError::Option)?;
    Ok(Texture::from_image(
        texture_context,
        &image,
        &TextureSettings::new(),
    )?)
}
