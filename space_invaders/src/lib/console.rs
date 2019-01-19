extern crate glutin_window;
extern crate graphics;
extern crate intel8080cpu;
extern crate opengl_graphics;
extern crate piston;

use self::glutin_window::GlutinWindow as Window;
use self::intel8080cpu::*;
use self::opengl_graphics::{GlGraphics, OpenGL};
use self::piston::event_loop::*;
use self::piston::input::*;
use self::piston::window::WindowSettings;
use super::failure::Error;
use super::io_devices::*;
use super::screen::{GameScreen, Screen};
use super::timer::Timer;
use super::view::{View, WINDOW_HEIGHT, WINDOW_WIDTH};
use super::ConsoleError;

const FPS: f64 = 60.0;
const SCREEN_INTERRUPTIONS_INTERVAL: f64 = (1.0 / FPS * 1000.0) / 2.0;
const OPEN_GL: OpenGL = OpenGL::V3_2;
pub(crate) const FRAME_BUFFER_ADDRESS: usize = 0x2400;
pub(crate) const FRAME_BUFFER_SIZE: usize = 0x1C00;

pub struct ConsoleOptions<'a> {
    has_audio: bool,
    folder: &'a str,
    memory: [u8; ROM_MEMORY_LIMIT],
}

impl<'a> ConsoleOptions<'a> {
    pub fn new(memory: [u8; ROM_MEMORY_LIMIT], folder: &'a str) -> ConsoleOptions<'a> {
        ConsoleOptions {
            folder,
            memory,
            has_audio: true,
        }
    }

    pub fn with_audio(mut self, has_audio: bool) -> ConsoleOptions<'a> {
        self.has_audio = has_audio;
        self
    }
}

pub struct Console<'a> {
    cpu: Intel8080Cpu<'a>,
    cycles_left: i64,
    gl: GlGraphics,
    keypad_controller: KeypadController,
    prev_interruption: u8,
    screen: Box<Screen>,
    timer: Timer,
    view: View,
    window: Window,
}

impl<'a> Console<'a> {
    pub fn new(options: ConsoleOptions) -> Result<Console, Error> {
        let timer = Timer::new(SCREEN_INTERRUPTIONS_INTERVAL);
        let keypad_controller = KeypadController::new();
        let cpu = Console::create_cpu(&keypad_controller, options)?;
        let screen = Box::new(GameScreen::new());
        let window = Console::create_window()?;
        let gl = GlGraphics::new(OPEN_GL);
        let view = View::new();

        Ok(Console {
            cpu,
            cycles_left: 0,
            gl,
            keypad_controller,
            prev_interruption: 2,
            screen,
            timer,
            view,
            window,
        })
    }

    fn create_cpu<'b>(
        keypad_controller: &KeypadController,
        options: ConsoleOptions,
    ) -> Result<Intel8080Cpu<'b>, Error> {
        let mut cpu = Intel8080Cpu::new(options.memory);
        let shift_writer = ExternalShiftWriter::new();
        let offset_writer = ExternalShiftOffsetWriter::new();
        let shift_reader = ExternalShiftReader::new(&shift_writer, &offset_writer);

        cpu.add_input_device(0, Box::new(DummyInputDevice { value: 1 }));
        cpu.add_input_device(1, Box::new(KeypadInput::new(keypad_controller)));
        cpu.add_input_device(2, Box::new(DummyInputDevice { value: 1 }));
        cpu.add_input_device(3, Box::new(shift_reader));
        cpu.add_output_device(2, Box::new(offset_writer));
        cpu.add_output_device(4, Box::new(shift_writer));
        cpu.add_output_device(6, Box::new(DummyOutputDevice {}));
        if options.has_audio {
            cpu.add_output_device(3, Box::new(SoundPort1::new(options.folder)?));
            cpu.add_output_device(5, Box::new(SoundPort2::new(options.folder)?));
        } else {
            cpu.add_output_device(3, Box::new(DummyOutputDevice {}));
            cpu.add_output_device(5, Box::new(DummyOutputDevice {}));
        }
        Ok(cpu)
    }

    fn create_window() -> Result<Window, Error> {
        WindowSettings::new("Space Invaders", [WINDOW_WIDTH, WINDOW_HEIGHT])
            .opengl(OPEN_GL)
            .exit_on_esc(true)
            .srgb(false)
            .build()
            .map_err(|e| Error::from(ConsoleError::CantCreateWindow { msg: e }))
    }

    pub fn start(&mut self) -> Result<(), Error> {
        self.timer.reset();
        let mut events = Events::new(EventSettings::new().ups(1000).max_fps(60));
        while let Some(e) = events.next(&mut self.window) {
            if self.cpu.is_done() {
                break;
            }
            if let Some(r) = e.render_args() {
                self.view.render(&r, &mut self.gl);
            }

            if let Some(u) = e.update_args() {
                self.update(&u)?;
            }

            if let Some(Button::Keyboard(key)) = e.press_args() {
                self.keypad_controller.key_pressed(key);
            }

            if let Some(Button::Keyboard(key)) = e.release_args() {
                self.keypad_controller.key_released(key);
            }
        }
        Ok(())
    }

    fn update(&mut self, args: &UpdateArgs) -> Result<(), Error> {
        self.timer.update_last_check();
        if self.timer.should_trigger() && self.cpu.interruptions_enabled {
            self.prev_interruption = if self.prev_interruption == 1 {
                let frame_buffer = &self.cpu.memory
                    [FRAME_BUFFER_ADDRESS..(FRAME_BUFFER_ADDRESS + FRAME_BUFFER_SIZE)];
                self.screen.on_full_screen(frame_buffer);
                2
            } else {
                let frame_buffer = &self.cpu.memory
                    [FRAME_BUFFER_ADDRESS..(FRAME_BUFFER_ADDRESS + FRAME_BUFFER_SIZE)];
                self.screen.on_mid_screen(frame_buffer);
                1
            };
            self.view.update_image(self.screen.get_pixels());
            self.cpu.execute_instruction(&Intel8080Instruction::Rst {
                byte: self.prev_interruption,
            })?;
        }
        let mut cycles_to_run = (args.dt * (HERTZ as f64)) as i64 + self.cycles_left;
        while cycles_to_run > 0 {
            cycles_to_run -= self.cpu.execute()? as i64;
        }
        self.cycles_left = cycles_to_run;
        Ok(())
    }
}
