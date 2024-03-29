use std::time::{Duration, Instant};

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
const WIDTH_FACTORED: u32 = WIDTH * SCALE_FACTOR;
const HEIGHT_FACTORED: u32 = HEIGHT * SCALE_FACTOR;

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

const TARGET_TIME_AT_500: Duration = Duration::from_millis(2);
const TARGET_TIME_AT_60: Duration = Duration::from_nanos(16666670);

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

    system.memory[..FONTSET.len()].copy_from_slice(&FONTSET[..]);

    system
  }

  fn keypad_event(&mut self, index: usize, state: bool) {
    self.keypad[index] = state;
    if self.waiting_key && state {
      self.waiting_key = false;
      self.v[self.get_x(
        (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16),
      )] = index as u8;
      self.pc += 2;
    }
  }

  pub fn run(mut self) -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title(env!("CARGO_PKG_NAME"))
      .with_inner_size(LogicalSize::new(WIDTH_FACTORED, HEIGHT_FACTORED))
      .build(&event_loop)?;

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = futures::executor::block_on(instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: Default::default(),
        compatible_surface: Some(&surface),
      },
    ))
    .unwrap();
    let (device, queue) = futures::executor::block_on(adapter.request_device(
      &wgpu::DeviceDescriptor {
        label: None,
        features: Default::default(),
        limits: Default::default(),
      },
      None,
    ))?;

    let mut flags = wgpu::ShaderFlags::VALIDATION;
    match adapter.get_info().backend {
      wgpu::Backend::Metal | wgpu::Backend::Vulkan | wgpu::Backend::Gl => {
        flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION
      }
      _ => (), //TODO
    }
    let shader_module =
      device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
          include_str!("./shader.wgsl"),
        )),
        flags,
      });

    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: 8 * 32,
      usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
      mapped_at_creation: false,
    });

    let bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStage::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            min_binding_size: None,
            has_dynamic_offset: false,
          },
          count: None,
        }],
      });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: storage_buffer.as_entire_binding(),
      }],
    });

    let pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
      });
    let render_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &shader_module,
          entry_point: "vs_main",
          buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
          module: &shader_module,
          entry_point: "fs_main",
          targets: &[wgpu::TextureFormat::Bgra8Unorm.into()],
        }),
      });

    let mut swap_chain_descriptor = wgpu::SwapChainDescriptor {
      usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
      format: wgpu::TextureFormat::Bgra8Unorm,
      width: WIDTH_FACTORED,
      height: HEIGHT_FACTORED,
      present_mode: wgpu::PresentMode::Fifo,
    };
    let mut swap_chain =
      device.create_swap_chain(&surface, &swap_chain_descriptor);

    let mut last_update_op = Instant::now();
    let mut last_update_redraw = Instant::now();
    event_loop.run(move |event, _, control_flow| {
      let _ = (
        &instance,
        &adapter,
        &shader_module,
        &bind_group,
        &pipeline_layout,
      );

      match event {
        Event::MainEventsCleared => {
          if last_update_op.elapsed() >= TARGET_TIME_AT_500 {
            last_update_op = Instant::now();

            if !self.waiting_key {
              self
                .execute_opcode(
                  (self.memory[self.pc] as u16) << 8
                    | (self.memory[self.pc + 1] as u16),
                )
                .unwrap();
            }
          }

          if last_update_redraw.elapsed() >= TARGET_TIME_AT_60 {
            last_update_redraw = Instant::now();

            if self.delay_timer > 0 {
              self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
              if self.sound_timer == 1 {
                println!("BEEP!");
              }
              self.sound_timer -= 1;
            }

            window.request_redraw();
          }
        }
        Event::RedrawRequested(_) => {
          let bit_data = {
            let mut res = [0u64; 32];

            for (index, states) in self.display.states.iter().enumerate() {
              let mut num = res[index];
              for state in states {
                num <<= 1;
                if *state {
                  num |= 0b1;
                }
              }
              res[index] = num;
            }

            res
          };

          let frame = swap_chain.get_current_frame().unwrap();
          let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
              label: None,
            });

          queue.write_buffer(
            &storage_buffer,
            0,
            bytemuck::cast_slice(&bit_data),
          );

          {
            let mut render_pass =
              encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                  view: &frame.output.view,
                  resolve_target: None,
                  ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                  },
                }],
                depth_stencil_attachment: None,
              });

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
          }

          queue.submit(Some(encoder.finish()));
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
              self.keypad_event(0xA, is_pressed);
            }
          }
          WindowEvent::Destroyed | WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
          }
          WindowEvent::Resized(size) => {
            swap_chain_descriptor.width = size.width;
            swap_chain_descriptor.height = size.height;
            swap_chain =
              device.create_swap_chain(&surface, &swap_chain_descriptor);
          }
          _ => {}
        },
        _ => {}
      };
    })
  }

  pub fn load_program(&mut self, data: &[u8]) {
    self.memory[0x200..(0x200 + data.len())].copy_from_slice(data);
  }

  fn execute_opcode(&mut self, opcode: u16) -> Result<(), anyhow::Error> {
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
        self.v[0x0f] = (self.v[y] > self.v[x]) as u8;
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
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
      _ => anyhow::bail!("Op not implemented: {:X}", opcode),
    };

    match pc {
      ProgramCounter::Next => self.pc += 0x2,
      ProgramCounter::Skip => self.pc += 0x4,
      ProgramCounter::Jump(addr) => self.pc = addr as usize,
      ProgramCounter::None => {}
    };

    Ok(())
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
