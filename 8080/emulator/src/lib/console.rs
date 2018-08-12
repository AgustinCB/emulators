extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;

use self::glutin_window::GlutinWindow as Window;
use self::opengl_graphics::{ GlGraphics, OpenGL };
use self::piston::window::WindowSettings;
use self::piston::event_loop::*;
use self::piston::input::*;
use super::cpu::{Cpu, Instruction, ROM_MEMORY_LIMIT};
use super::io_devices::*;
use super::screen::{Screen, TermScreen};
use super::timer::Timer;

const SCREEN_INTERRUPTIONS_INTERVAL: f64 = (1.0/60.0*1000.0)/2.0;
const OPEN_GL: OpenGL = OpenGL::V3_2;
const WINDOW_HEIGHT: u32 = 480;
const WINDOW_WIDTH: u32 = 480;

pub struct Console<'a> {
    cpu: Cpu<'a>,
    cycles_left: i64,
    gl: GlGraphics,
    keypad_controller: KeypadController,
    prev_interruption: u8,
    screen: Box<Screen>,
    timer: Timer,
    window: Window,
}

impl<'a> Console<'a> {
    pub fn new<'b>(memory: [u8; ROM_MEMORY_LIMIT]) -> Result<Console<'b>, String> {
        let timer = Timer::new(SCREEN_INTERRUPTIONS_INTERVAL);
        let keypad_controller = KeypadController::new();
        let cpu = Console::create_cpu(memory, &keypad_controller);
        let screen = Box::new(TermScreen::new(&cpu.memory));
        let window = Console::create_window()?;
        let gl = GlGraphics::new(OPEN_GL);

        Ok(Console {
            cpu,
            cycles_left: 0,
            gl,
            keypad_controller,
            prev_interruption: 2,
            screen,
            timer,
            window,
        })
    }

    fn create_cpu<'b>(memory: [u8; ROM_MEMORY_LIMIT], keypad_controller: &KeypadController) -> Cpu<'b> {
        let mut cpu = Cpu::new(memory);
        let shift_writer = ExternalShiftWriter::new();
        let offset_writer = ExternalShiftOffsetWriter::new();
        let shift_reader = ExternalShiftReader::new(&shift_writer, &offset_writer);

        cpu.add_input_device(0, Box::new(DummyInputDevice { value: 1 }));
        cpu.add_input_device(1, Box::new(KeypadInput::new(keypad_controller)));
        cpu.add_input_device(2, Box::new(DummyInputDevice { value: 1 }));
        cpu.add_input_device(3, Box::new(shift_reader));
        cpu.add_output_device(2, Box::new(offset_writer));
        cpu.add_output_device(3, Box::new(DummyOutputDevice{}));
        cpu.add_output_device(4, Box::new(shift_writer));
        cpu.add_output_device(5, Box::new(DummyOutputDevice{}));
        cpu.add_output_device(6, Box::new(DummyOutputDevice{}));
        cpu
    }

    fn create_window() -> Result<Window, String> {
        WindowSettings::new(
            "Space Invaders",
            [WINDOW_WIDTH, WINDOW_HEIGHT]
        )
            .opengl(OPEN_GL)
            .exit_on_esc(true)
            .build()
    }

    pub fn start(&mut self) {
        self.timer.reset();
        let mut events = Events::new(
            EventSettings::new().ups(1000).max_fps(60));
        while let Some(e) = events.next(&mut self.window) {
            if self.cpu.is_done() {
                break;
            }
            if let Some(r) = e.render_args() {
                self.render(&r);
            }

            if let Some(u) = e.update_args() {
                self.update(&u);
            }

            if let Some(Button::Keyboard(key)) = e.press_args() {
                self.keypad_controller.key_pressed(key);
            }

            if let Some(Button::Keyboard(key)) = e.release_args() {
                self.keypad_controller.key_released(key);
            }
        }
    }

    fn render(&mut self, _args: &RenderArgs) {

    }

    fn update(&mut self, args: &UpdateArgs) {
        self.timer.update_last_check();
        if self.timer.should_trigger() && self.cpu.interruptions_enabled {
            self.prev_interruption = if self.prev_interruption == 1 {
                self.screen.on_full_screen();
                2
            } else {
                self.screen.on_mid_screen();
                1
            };
            self.cpu.execute_instruction(Instruction::Rst {
                value: self.prev_interruption
            });
        }
        let mut cycles_to_run = ((args.dt * 1000.0) as i64 * 2) as i64 + self.cycles_left;
        while cycles_to_run > 0 {
            cycles_to_run -= self.cpu.execute() as i64;
        }
        self.cycles_left = cycles_to_run;
    }
}