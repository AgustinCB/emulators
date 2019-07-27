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
use self::piston_window::{Glyphs, PistonWindow};
use super::screen::{ScreenLayout, SCREEN_HEIGHT, SCREEN_WIDTH};

pub(crate) const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32;
pub(crate) const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32;

pub struct View {
    left_menu_visible: bool,
    image: RgbaImage,
    texture: Texture,
    glyphs: Glyphs,
}

impl View {
    pub fn new(debug: bool, glyphs: Glyphs) -> View {
        let image = ImageBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let texture = Texture::from_image(&image.convert(), &TextureSettings::new());
        let left_menu_visible = debug;
        View {
            image,
            glyphs,
            left_menu_visible,
            texture,
        }
    }

    pub fn render(&mut self, event: &Event, args: &RenderArgs, window: &mut PistonWindow) {
        use self::graphics::*;
        let (x, y) = (
            args.window_size[0] / 2f64 - (SCREEN_WIDTH / 2) as f64,
            args.window_size[1] / 2f64 - (SCREEN_HEIGHT / 2) as f64,
        );
        let mut context = window.create_texture_context();
        window.draw_2d(event, |c, gl, device| {
            let transform = c.transform.trans(x, y);
            clear([0.0, 0.0, 0.0, 1.0], gl);
            let img =
                GfxTexture::from_image(&mut context, &self.image, &TextureSettings::new()).unwrap();
            image(&img, transform, gl);
            if self.left_menu_visible {
                let menu_transform = transform.trans(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
                clear([0.0, 0.0, 0.0, 1.0], gl);
                text::Text::new_color([0.0, 1.0, 0.0, 1.0], 32)
                    .draw(
                        "This is a menu",
                        &mut self.glyphs,
                        &c.draw_state,
                        menu_transform,
                        gl,
                    )
                    .unwrap();

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
