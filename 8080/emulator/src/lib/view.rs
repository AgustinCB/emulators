extern crate graphics;
extern crate image as im;
extern crate opengl_graphics;
extern crate piston;

use self::graphics::{clear, image, rectangle};
use self::im::{ConvertBuffer, GrayImage, ImageBuffer, Luma};
use self::opengl_graphics::{GlGraphics, Texture, TextureSettings};
use self::piston::input::*;
use super::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenLayout};

pub(crate) const WINDOW_HEIGHT: u32 = 480;
pub(crate) const WINDOW_WIDTH: u32 = 480;
const VIEW_AREA: [f64; 4] = [
    ((WINDOW_HEIGHT as usize - SCREEN_HEIGHT)/2) as f64,
    ((WINDOW_WIDTH as usize - SCREEN_WIDTH)/2) as f64,
    (SCREEN_WIDTH + 2) as f64,
    (SCREEN_HEIGHT + 2)  as f64
];

pub(crate) struct View {
    image: GrayImage,
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
        gl.draw(args.viewport(), |c, gl| {
            clear([0.0, 0.0, 0.0, 1.0], gl);
            rectangle([1.0, 0.0, 0.0, 1.0],VIEW_AREA, c.transform,gl);
            image(&self.texture, c.transform, gl);
        });
        Ok(())
    }

    pub fn update_image(&mut self, pixels: &ScreenLayout) {
        for line in 0..pixels.len() {
            for column in 0..pixels[line].len() {
                if pixels[line][column] {
                    self.image.put_pixel(line as u32, column as u32, Luma([1]));
                } else {
                    self.image.put_pixel(line as u32, column as u32, Luma([0]));
                }
            }
        }
        self.texture.update(&self.image.convert());
    }
}