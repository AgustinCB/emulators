extern crate graphics;
extern crate image as im;
extern crate opengl_graphics;
extern crate piston;

use self::im::{ConvertBuffer, RgbaImage, ImageBuffer, Rgba};
use self::opengl_graphics::{GlGraphics, Texture, TextureSettings};
use self::piston::input::*;
use super::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenLayout};

pub(crate) const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32;
pub(crate) const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32;

pub(crate) struct View {
    image: RgbaImage,
    texture: Texture,
}

impl View {
    pub fn new() -> View {
        let image = ImageBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let texture = Texture::from_image(&image.convert(), &TextureSettings::new());
        View{
            image,
            texture,
        }
    }

    pub fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) -> Result<(), String> {
        use self::graphics::*;
        let (x, y) = (
            (args.width/2) as f64 - (SCREEN_WIDTH / 2) as f64,
            (args.height/2) as f64 - (SCREEN_HEIGHT / 2) as f64);
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform.trans(x, y);
            clear([0.0, 0.0, 0.0, 1.0], gl);
            image(&self.texture, transform, gl);
        });
        Ok(())
    }

    pub fn update_image(&mut self, pixels: &ScreenLayout) {
        for line in 0..pixels.len() {
            for column in 0..pixels[line].len() {
                let pixel = if pixels[line][column] {
                    [255; 4]
                } else {
                    [0, 0, 0, 255]
                };
                self.image.put_pixel(column as u32, line as u32, Rgba(pixel));
            }
        }
        self.texture.update(&self.image);
    }
}