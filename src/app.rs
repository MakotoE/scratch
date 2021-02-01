use super::*;
use crate::file::ScratchFile;
use crate::interface::Interface;
use conrod_core::text::GlyphCache;
use conrod_core::widget::{Canvas, FileNavigator};
use conrod_core::{Borderable, Colorable, Positionable, Sizeable, Theme, Widget};
use gfx_graphics::GfxGraphics;
use graphics::math::Matrix2d;
use graphics::rectangle::Shape;
use graphics::{Context, DrawState, Rectangle};
use piston_window::texture::UpdateTexture;
use piston_window::{
    G2d, G2dTexture, G2dTextureContext, OpenGL, OpenGLWindow, PistonWindow, RenderEvent, Size,
    Texture, TextureSettings, UpdateEvent, Window, WindowSettings,
};
use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::path::Path;
use tokio::task::spawn_blocking;

pub async fn app(file_path: &Path) -> Result<()> {
    const PAGE_SIZE: Size = Size {
        width: 520.0,
        height: 480.0,
    };

    let mut window: PistonWindow = WindowSettings::new("Scratch", PAGE_SIZE)
        .graphics_api(OpenGL::V3_2)
        .samples(8)
        .vsync(true)
        .resizable(false)
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

    let scratch_file = ScratchFile::parse(BufReader::new(File::open(file_path)?))?;

    let green_flag_id = image_map.insert(image_texture(
        &mut texture_context,
        Path::new("assets/green_flag.svg"),
    )?);

    let stop_image_id = image_map.insert(image_texture(
        &mut texture_context,
        Path::new("assets/stop.svg"),
    )?);

    let id_generator = ui.widget_id_generator();
    let mut interface = Interface::new(
        &mut texture_context,
        scratch_file,
        interface::Ids::new(id_generator),
        green_flag_id,
        stop_image_id,
    )
    .await?;

    let mut character_cache = window.load_font("assets/Roboto-Regular.ttf").unwrap();

    let mut text_vertex_data: Vec<u8> = Vec::new();

    while let Some(event) = window.next() {
        let size = window.size();
        if let Some(e) = conrod_piston::event::convert(event.clone(), size.width, size.height) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui_cell = ui.set_widgets();
            interface.widgets(&mut ui_cell);
        });

        if let Some(args) = event.render_args() {
            window.window.make_current();

            let device = &mut window.device;
            let mut graphics = GfxGraphics::new(
                &mut window.encoder,
                &window.output_color,
                &window.output_stencil,
                &mut window.g2d,
            );
            let mut context = Context::new_viewport(args.viewport());
            if let Some(primitives) = ui.draw_if_changed() {
                draw_border(&context.draw_state, context.transform, &mut graphics);

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
                    &mut graphics,
                    &mut text_texture_cache,
                    &mut glyph_cache,
                    &image_map,
                    cache_queued_glyphs,
                    |img| img,
                );

                interface
                    .draw_2d(&mut context, &mut graphics, &mut character_cache)
                    .await
                    .unwrap();

                // Might need to graphics.flush_colored()
                window.encoder.flush(device);
                texture_context.encoder.flush(device);
                character_cache.factory.encoder.flush(device);
            }
        }
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
    rectangle.draw([0.0, 410.0, 520.0, 480.0], draw_state, transform, graphics);
    // Left
    rectangle.draw([0.0, 50.0, 20.0, 410.0], draw_state, transform, graphics);
    // Right
    rectangle.draw([500.0, 50.0, 480.0, 410.0], draw_state, transform, graphics);
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
        tiny_skia::Pixmap::new(size.width() * 2, size.height() * 2).ok_or(ScratchError::Option)?;
    let width = pixmap.width();
    let height = pixmap.height();

    resvg::render(&tree, usvg::FitTo::Zoom(2.0), pixmap.as_mut()).ok_or(ScratchError::Option)?;
    let image =
        image::ImageBuffer::from_raw(width, height, pixmap.take()).ok_or(ScratchError::Option)?;
    Ok(Texture::from_image(
        texture_context,
        &image,
        &TextureSettings::new(),
    )?)
}
