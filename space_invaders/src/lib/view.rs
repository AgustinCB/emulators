extern crate gfx_texture;
extern crate graphics;
extern crate image as im;
extern crate intel8080cpu;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;

use self::gfx_texture::Texture as GfxTexture;
use self::im::{ConvertBuffer, ImageBuffer, Rgba, RgbaImage};
use self::intel8080cpu::Intel8080Instruction;
use self::opengl_graphics::{Texture, TextureSettings};
use self::piston::{Event, RenderArgs};
use self::piston_window::*;
use super::screen::{ScreenLayout, SCREEN_HEIGHT, SCREEN_WIDTH};

pub(crate) const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32;
pub(crate) const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32;
const PAUSE_BUTTON_HEIGHT: usize = 50usize;
const PAUSE_BUTTON_WIDTH: usize = 50usize;
const PAUSE_BUTTON: [[bool; PAUSE_BUTTON_WIDTH]; PAUSE_BUTTON_HEIGHT] = [
    [true; 50],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true, true, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true, true, true, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true, true, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, true, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, true, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, true, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, true,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, true, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, true, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, true, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, true, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, true, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, true,
    ],
    [true; 50],
];

fn update_image(pixels: &[&[bool]], image: &mut RgbaImage, texture: &mut Texture) {
    for (line, row) in pixels.iter().enumerate() {
        for (column, drawn_pixel) in row.iter().enumerate() {
            let pixel = if *drawn_pixel {
                [255; 4]
            } else {
                [0, 0, 0, 255]
            };
            image.put_pixel(column as u32, line as u32, Rgba(pixel));
        }
    }
    texture.update(&image);
}

pub struct View {
    glyphs: Glyphs,
    image: RgbaImage,
    left_menu_visible: bool,
    pause_texture: G2dTexture,
    pause_position: [f64; 2],
    texture: Texture,
    texture_context: G2dTextureContext,
}

impl View {
    pub fn new(debug: bool, glyphs: Glyphs, mut texture_context: G2dTextureContext) -> View {
        let image = ImageBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let mut pause_image =
            ImageBuffer::new(PAUSE_BUTTON_WIDTH as u32, PAUSE_BUTTON_HEIGHT as u32);
        let texture = Texture::from_image(&image.convert(), &TextureSettings::new());
        let mut pause_texture =
            Texture::from_image(&pause_image.convert(), &TextureSettings::new());
        let left_menu_visible = debug;
        let p = &PAUSE_BUTTON
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<&[bool]>>();
        update_image(p.as_ref(), &mut pause_image, &mut pause_texture);
        let pause_img =
            GfxTexture::from_image(&mut texture_context, &pause_image, &TextureSettings::new())
                .unwrap();
        View {
            glyphs,
            image,
            left_menu_visible,
            pause_texture: pause_img,
            pause_position: [0f64; 2],
            texture,
            texture_context,
        }
    }

    pub fn render<'a, I: Iterator<Item = &'a Intel8080Instruction>>(
        &mut self,
        event: &Event,
        args: &RenderArgs,
        window: &mut PistonWindow,
        instructions: I,
    ) {
        use self::graphics::*;
        self.pause_position[0] =
            args.window_size[0] / 2f64 - (SCREEN_WIDTH / 2) as f64 + SCREEN_WIDTH as f64;
        self.pause_position[1] = args.window_size[1] / 2f64 - (SCREEN_HEIGHT / 2) as f64;
        let (x, y) = (
            args.window_size[0] / 2f64 - (SCREEN_WIDTH / 2) as f64,
            args.window_size[1] / 2f64 - (SCREEN_HEIGHT / 2) as f64,
        );
        window.draw_2d(event, |c, gl, device| {
            let transform = c.transform.trans(x, y);
            clear([0.0, 0.0, 0.0, 1.0], gl);
            let img = GfxTexture::from_image(
                &mut self.texture_context,
                &self.image,
                &TextureSettings::new(),
            )
            .unwrap();
            image(&img, transform, gl);
            if self.left_menu_visible {
                let (x, y) = (SCREEN_WIDTH as f64, 0f64);
                let menu_transform = transform.trans(x, y);
                image(&self.pause_texture, menu_transform, gl);
                let mut instruction_transform = menu_transform.trans(0.0, 55.0);
                for instruction in instructions {
                    instruction_transform = instruction_transform.trans(0.0, 20.0);
                    text::Text::new_color([0.0, 1.0, 0.0, 1.0], 15)
                        .draw(
                            instruction.to_string().as_str(),
                            &mut self.glyphs,
                            &c.draw_state,
                            instruction_transform,
                            gl,
                        )
                        .unwrap();
                }
                // Update glyphs before rendering.
                self.glyphs.factory.encoder.flush(device);
            }
        });
    }

    pub fn update_image(&mut self, pixels: &ScreenLayout) {
        let p = &pixels.iter().map(|a| a.as_ref()).collect::<Vec<&[bool]>>();
        update_image(p.as_ref(), &mut self.image, &mut self.texture)
    }

    pub fn is_in_pause_button(&self, position: [f64; 2]) -> bool {
        position[0] >= self.pause_position[0]
            && position[0] < (self.pause_position[0] + PAUSE_BUTTON_WIDTH as f64)
            && position[1] >= self.pause_position[1]
            && position[1] < (self.pause_position[1] + PAUSE_BUTTON_HEIGHT as f64)
    }
}
