use std::{thread, time};

use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use rand::Rng;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::display;

const SCALE_FACTOR: u32 = 10;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

static FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

enum ProgramCounter {
    Next,
    Skip,
    Jump(u16),
    None,
}

pub struct System {
    memory: [u8; 4096],
    v: [u8; 16],
    i: usize,
    pc: usize,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    display: display::Display,
    keypad: [bool; 16],
    waiting_key: bool,
}

impl System {
    pub fn new() -> System {
        let mut system = System {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: display::Display::new(),
            keypad: [false; 16],
            waiting_key: false,
        };

        system.memory[..FONTSET.len()].clone_from_slice(&FONTSET[..]);

        system
    }

    fn keypad_event(&mut self, index: usize, state: bool) {
        self.keypad[index] = state;
        if self.waiting_key && state {
            self.waiting_key = false;
            self.v[self
                .get_x((self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16))] =
                index as u8;
            self.pc += 2;
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(
                (SCALE_FACTOR * WIDTH) as f64,
                (SCALE_FACTOR * HEIGHT) as f64,
            );
            WindowBuilder::new()
                .with_title(env!("CARGO_PKG_NAME"))
                .with_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };
        let mut pixels = {
            let surface = Surface::create(&window);
            let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };
        let dur = time::Duration::from_millis(2);
        event_loop.run(move |event, _, control_flow| {
            if !self.waiting_key {
                self.execute_opcode(
                    (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16),
                );
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
                if self.sound_timer > 0 {
                    if self.sound_timer == 1 {
                        println!("BEEP!");
                    }
                    self.sound_timer -= 1;
                }
            }

            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared => {
                    thread::sleep(dur);
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    for (i, pixel) in pixels.get_frame().chunks_exact_mut(4).enumerate() {
                        let y = i / WIDTH as usize;
                        let x = i % WIDTH as usize;

                        let rgba = if self.display.arr[y][x] {
                            [0xFF, 0xFF, 0xFF, 0xFF]
                        } else {
                            [0x00, 0x00, 0x00, 0xFF]
                        };

                        pixel.copy_from_slice(&rgba);
                    }

                    if pixels
                        .render()
                        .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                        .is_err()
                    {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        let is_pressed = input.state == ElementState::Pressed;
                        match input.virtual_keycode.unwrap() {
                            VirtualKeyCode::Escape => {
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                            VirtualKeyCode::Key1 => self.keypad_event(0x1, is_pressed),
                            VirtualKeyCode::Key2 => self.keypad_event(0x2, is_pressed),
                            VirtualKeyCode::Key3 => self.keypad_event(0x3, is_pressed),
                            VirtualKeyCode::Key4 => self.keypad_event(0xC, is_pressed),
                            VirtualKeyCode::Q => self.keypad_event(0x4, is_pressed),
                            VirtualKeyCode::W => self.keypad_event(0x5, is_pressed),
                            VirtualKeyCode::E => self.keypad_event(0x6, is_pressed),
                            VirtualKeyCode::R => self.keypad_event(0xD, is_pressed),
                            VirtualKeyCode::A => self.keypad_event(0x7, is_pressed),
                            VirtualKeyCode::S => self.keypad_event(0x8, is_pressed),
                            VirtualKeyCode::D => self.keypad_event(0x9, is_pressed),
                            VirtualKeyCode::F => self.keypad_event(0xE, is_pressed),
                            VirtualKeyCode::X => self.keypad_event(0x0, is_pressed),
                            VirtualKeyCode::C => self.keypad_event(0xB, is_pressed),
                            VirtualKeyCode::V => self.keypad_event(0xF, is_pressed),
                            _ => {}
                        }
                        if input.scancode == 6 {
                            self.keypad_event(0xA, is_pressed)
                        }
                    }
                    WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        pixels.resize(size.width, size.height);
                    }
                    _ => {}
                },
                _ => {}
            };
        });
    }

    pub fn load_program(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            if (0x200 + i) < 0x1000 {
                self.memory[0x200 + i] = byte;
            } else {
                break;
            }
        }
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let (a, x, y, n) = (
            0xf & opcode >> 12,
            self.get_x(opcode),
            self.get_y(opcode),
            self.get_n(opcode),
        );
        let nn = self.get_nn(opcode);
        let nnn = self.get_nnn(opcode);

        let pc = match (a, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => {
                self.display.clear();
                ProgramCounter::Next
            }
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                ProgramCounter::Jump(self.stack[self.sp])
            }
            (0x1, _, _, _) => ProgramCounter::Jump(nnn),
            (0x2, _, _, _) => {
                self.stack[self.sp] = self.pc as u16 + 2;
                self.sp += 1;
                ProgramCounter::Jump(nnn)
            }
            (0x3, _, _, _) => {
                if self.v[x] == nn {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }
            (0x4, _, _, _) => {
                if self.v[x] != nn {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }
            (0x5, _, _, 0x0) => {
                if self.v[x] == self.v[y] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }
            (0x6, _, _, _) => {
                self.v[x] = nn;
                ProgramCounter::Next
            }
            (0x7, _, _, _) => {
                self.v[x] = self.v[x].overflowing_add(nn).0;
                ProgramCounter::Next
            }
            (0x8, _, _, 0x0) => {
                self.v[x] = self.v[y];
                ProgramCounter::Next
            }
            (0x8, _, _, 0x1) => {
                self.v[x] |= self.v[y];
                ProgramCounter::Next
            }
            (0x8, _, _, 0x2) => {
                self.v[x] &= self.v[y];
                ProgramCounter::Next
            }
            (0x8, _, _, 0x3) => {
                self.v[x] ^= self.v[y];
                ProgramCounter::Next
            }
            (0x8, _, _, 0x4) => {
                let calc = self.v[x].overflowing_add(self.v[y]);
                self.v[0xF] = calc.1 as u8;
                self.v[x] = calc.0;
                ProgramCounter::Next
            }
            (0x8, _, _, 0x5) => {
                self.v[0xF] = (self.v[x] > self.v[y]) as u8;
                self.v[x] = self.v[x].overflowing_sub(self.v[y]).0;
                ProgramCounter::Next
            }
            (0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[x] & 1;
                self.v[x] >>= 1;
                ProgramCounter::Next
            }
            (0x8, _, _, 0x7) => {
                let calc = self.v[y].overflowing_sub(self.v[x]);
                self.v[0xF] = calc.1 as u8;
                self.v[x] = calc.0;
                ProgramCounter::Next
            }
            (0x8, _, _, 0xE) => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
                ProgramCounter::Next
            }
            (0x9, _, _, 0x0) => {
                if self.v[x] != self.v[y] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }
            (0xA, _, _, _) => {
                self.i = nnn as usize;
                ProgramCounter::Next
            }
            (0xB, _, _, _) => ProgramCounter::Jump(nnn + self.v[0] as u16),
            (0xC, _, _, _) => {
                let mut rng = rand::thread_rng();
                self.v[x] = nn & rng.gen::<u8>();
                ProgramCounter::Next
            }
            (0xD, _, _, _) => {
                self.v[0xF] = self.display.draw(
                    self.v[x] as usize,
                    self.v[y] as usize,
                    &self.memory[self.i..(self.i + (n as usize))],
                ) as u8;
                ProgramCounter::Next
            }
            (0xE, _, 0x9, 0xE) => {
                if self.keypad[self.v[x] as usize] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }
            (0xE, _, 0xA, 0x1) => {
                if !self.keypad[self.v[x] as usize] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }
            (0xF, _, 0x0, 0x7) => {
                self.v[x] = self.delay_timer;
                ProgramCounter::Next
            }
            (0xF, _, 0x0, 0xA) => {
                self.waiting_key = true;
                ProgramCounter::None
            }
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.v[x];
                ProgramCounter::Next
            }
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.v[x];
                ProgramCounter::Next
            }
            (0xF, _, 0x1, 0xE) => {
                self.i += self.v[x] as usize;
                ProgramCounter::Next
            }
            (0xF, _, 0x2, 0x9) => {
                self.i = (self.v[x] as usize) * 5;
                ProgramCounter::Next
            }
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i] = self.v[x] / 100;
                self.memory[self.i + 1] = (self.v[x] % 100) / 10;
                self.memory[self.i + 2] = self.v[x] % 10;
                ProgramCounter::Next
            }
            (0xF, _, 0x5, 0x5) => {
                for i in 0..=x {
                    self.memory[self.i + i] = self.v[i]
                }
                ProgramCounter::Next
            }
            (0xF, _, 0x6, 0x5) => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i + i]
                }
                ProgramCounter::Next
            }
            _ => panic!("Op not implemented: {:X}", opcode),
        };

        match pc {
            ProgramCounter::Next => self.pc += 0x2,
            ProgramCounter::Skip => self.pc += 0x4,
            ProgramCounter::Jump(addr) => self.pc = addr as usize,
            ProgramCounter::None => {}
        };
    }

    fn get_x(&self, opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }
    fn get_y(&self, opcode: u16) -> usize {
        ((opcode & 0x0F0) >> 4) as usize
    }
    fn get_n(&self, opcode: u16) -> u8 {
        (opcode & 0x000F) as u8
    }
    fn get_nn(&self, opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }
    fn get_nnn(&self, opcode: u16) -> u16 {
        opcode & 0x0FFF
    }
}
