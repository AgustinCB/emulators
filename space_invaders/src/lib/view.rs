extern crate gfx_texture;
extern crate graphics;
extern crate image as im;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;

use self::gfx_texture::Texture as GfxTexture;
use self::im::{ConvertBuffer, ImageBuffer, Rgba, RgbaImage};
use self::opengl_graphics::{Texture, TextureSettings};
use self::piston::{Event, RenderArgs};
use self::piston_window::{G2dTexture, G2dTextureContext, Glyphs, PistonWindow};
use super::screen::{ScreenLayout, SCREEN_HEIGHT, SCREEN_WIDTH};

pub(crate) const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32;
pub(crate) const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32;

pub struct View {
    control_button: G2dTexture,
    left_menu_visible: bool,
    image: RgbaImage,
    texture: Texture,
    texture_context: G2dTextureContext,
    glyphs: Glyphs,
}

impl View {
    pub fn new(debug: bool, glyphs: Glyphs, control_button: G2dTexture, texture_context: G2dTextureContext) -> View {
        let image = ImageBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let texture = Texture::from_image(&image.convert(), &TextureSettings::new());
        let left_menu_visible = debug;
        View {
            control_button,
            image,
            glyphs,
            left_menu_visible,
            texture,
            texture_context,
        }
    }

    pub fn render(&mut self, event: &Event, args: &RenderArgs, window: &mut PistonWindow) {
        use self::graphics::*;
        let (x, y) = (
            args.window_size[0] / 2f64 - (SCREEN_WIDTH / 2) as f64,
            args.window_size[1] / 2f64 - (SCREEN_HEIGHT / 2) as f64,
        );
        window.draw_2d(event, |c, gl, device| {
            let transform = c.transform.trans(x, y);
            clear([0.0, 0.0, 0.0, 1.0], gl);
            let img =
                GfxTexture::from_image(&mut self.texture_context, &self.image, &TextureSettings::new()).unwrap();
            image(&img, transform, gl);
            if self.left_menu_visible {
                let (x, y) = (
                    SCREEN_WIDTH as f64,
                    50f64,
                );
                let menu_transform = transform.trans(x, y);
                image(&self.control_button, menu_transform, gl);
                /*
                text::Text::new_color([0.0, 1.0, 0.0, 1.0], 32)
                    .draw(
                        "This is a menu",
                        &mut self.glyphs,
                        &c.draw_state,
                        menu_transform,
                        gl,
                    )
                    .unwrap();
                */
                // Update glyphs before rendering.
                self.glyphs.factory.encoder.flush(device);
            }
        });
    }

    pub fn update_image(&mut self, pixels: &ScreenLayout) {
        for (line, row) in pixels.iter().enumerate() {
            for (column, drawn_pixel) in row.iter().enumerate() {
                let pixel = if *drawn_pixel {
                    [255; 4]
                } else {
                    [0, 0, 0, 255]
                };
                self.image
                    .put_pixel(column as u32, line as u32, Rgba(pixel));
            }
        }
        self.texture.update(&self.image);
    }
}
