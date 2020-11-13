use wasm_bindgen::prelude::*;

pub mod resources_manager;
pub mod virtual_machine;
pub mod opcodes;
pub mod video;
pub mod poly;
pub mod defines;
pub mod font;
pub mod game_strings;
pub mod utils;

use crate::defines::{FRAME_BUFFER_WIDTH, FRAME_BUFFER_HEIGHT};
use crate::resources_manager::{ResourcesManager, ResourceType};
use crate::virtual_machine::VirtualMachine;
use crate::video::Video;
use crate::defines::NUM_THREADS;
use crate::utils::{write_u16};
use crate::poly::{Poly, draw_poly_to_buffer};

const SHARED_MEMORY_SIZE: usize = 3 * 1024 * 1204; // 3Mb

#[wasm_bindgen]
pub struct AnotherWorldEngine {
  shared_memory: Vec<u8>,
  resources_manager: ResourcesManager,
  virtual_machine: VirtualMachine,
  video: Video
}

#[wasm_bindgen]
impl AnotherWorldEngine {
  pub fn new() -> AnotherWorldEngine {
    AnotherWorldEngine {
      shared_memory: vec![0; SHARED_MEMORY_SIZE],
      resources_manager: ResourcesManager::new(),
      virtual_machine: VirtualMachine::new(),
      video: Video::new()
    }
  }

  pub fn init(&mut self) {
    // this method is called after the game data has been copied to shared_memory
    self.resources_manager.init(&self.shared_memory);

    self.virtual_machine.init();
    self.vm_restart(0);

    self.build_resources_info();
  }

  pub fn get_screen_width(&self) -> u16 {
    FRAME_BUFFER_WIDTH
  }

  pub fn get_screen_height(&self) -> u16 {
    FRAME_BUFFER_HEIGHT
  }

  pub fn get_frame_buffer(&mut self) -> *const u8 {
    let page = self.video.get_screen_page();
    let palette_id = self.video.get_active_palette_id();
    let palette = &self.resources_manager.get_file(self.virtual_machine.palette_file_id)[palette_id as usize * 32..]; // * 32 = 16 colors * 2 bytes per color

    for i in 0..FRAME_BUFFER_HEIGHT * FRAME_BUFFER_WIDTH {
      let color_idx = page[i as usize];
      let c1 = palette[color_idx as usize * 2];
      let c2 = palette[color_idx as usize * 2 + 1];

      let r = (((c1 & 0x0f) << 2) | ((c1 & 0x0f) >> 2)) << 2;
      let g = (((c2 & 0xf0) >> 2) | ((c2 & 0xf0) >> 6)) << 2;
      let b = (((c2 & 0x0f) >> 2) | ((c2 & 0x0f) << 2)) << 2;

      self.shared_memory[i as usize * 4] = r;
      self.shared_memory[i as usize * 4 + 1] = g;
      self.shared_memory[i as usize * 4 + 2] = b;
      self.shared_memory[i as usize * 4 + 3] = 0;
    }

    self.shared_memory.as_ptr()
  }

  pub fn get_registers(&self) -> *const i16 {
    self.virtual_machine.registers.as_ptr()
  }

  pub fn set_game_data(&mut self, game_data: &[u8]) {
    self.shared_memory[..game_data.len()].clone_from_slice(game_data);
  }

  pub fn vm_step(&mut self) -> u32 {
    self.virtual_machine.step(&mut self.video, &self.resources_manager)
  }

  pub fn vm_restart(&mut self, level: u8) {
    self.virtual_machine.restart_level(level)
  }

  pub fn vm_get_current_pc(&self) -> u16 {
    self.virtual_machine.get_current_pc()
  }

  pub fn on_key_down(&mut self, key: u8) {
    self.virtual_machine.on_key_down(key);
  }

  pub fn on_key_up(&mut self, key: u8) {
    self.virtual_machine.on_key_up(key);
  }

  pub fn get_shared_memory_pointer(&self) -> *const u8 {
    self.shared_memory.as_ptr()
  }

  pub fn build_resources_info(&mut self) {
    self.shared_memory[0] = self.resources_manager.files.len() as u8;

    let mut idx = 1;

    for i in 0..self.resources_manager.files.len() {
      let ftype = self.resources_manager.files[i].ftype;

      self.shared_memory[idx] = ftype;
      idx += 1;

      let content_len = self.resources_manager.files[i].content.len() as u16;
      write_u16(&mut self.shared_memory, idx, content_len);
      idx += 2;

      if ftype == ResourceType::Palette as u8 {
        idx += self.build_palettes_info(i as u8, idx)
      } else if ftype == ResourceType::Script as u8 {
        idx += self.disassemble_script(i as u8, idx)
      } else {
        self.shared_memory[idx..idx + content_len as usize].copy_from_slice(self.resources_manager.get_file(i as u8));
        idx += content_len as usize;
      }
    }
  }

  pub fn build_threads_info(&mut self) {
    write_u16(&mut self.shared_memory, 0, self.virtual_machine.active_thread as u16);

    let mut idx = 2;

    for i in 0..NUM_THREADS {
      write_u16(&mut self.shared_memory, idx, self.virtual_machine.threads[i].pc);
      idx += 2;
    }
  }

  pub fn get_active_script_file_id(&self) -> u8 {
    self.virtual_machine.script_file_id
  }

  pub fn draw_poly(&mut self, file_id: u8, offset: u16, x: i16, y: i16, zoom: i16) {
    let size = (FRAME_BUFFER_WIDTH * FRAME_BUFFER_HEIGHT) as usize;
    let mut pages = [vec![0; size], vec![0; size], vec![0; size], vec![0; size]];
    let mut poly = Poly::new();

    draw_poly_to_buffer(&mut poly, self.resources_manager.get_file(file_id), offset, x, y, zoom, 0xff, &mut pages, 0);

    for i in 0..FRAME_BUFFER_WIDTH * FRAME_BUFFER_HEIGHT {
      self.shared_memory[i as usize] = pages[0][i as usize];
    }
  }

  fn build_palettes_info(&mut self, palettes_id: u8, idx: usize) -> usize {
    let palettes = self.resources_manager.get_file(palettes_id);
    let palettes_len = palettes.len();
    let mut my_idx = idx;
    let mut i: usize = 0;
    let mut num_palettes: u8 = 0;
    let mut num_colors = 0;

    my_idx += 1; // first byte = num palettes

    while i < palettes_len {
      // palettes are 16 colors and each color has 2 bytes
      let c1 = palettes[i];
      let c2 = palettes[i + 1];

      self.shared_memory[my_idx] = (((c1 & 0x0f) << 2) | ((c1 & 0x0f) >> 2)) << 2;
      self.shared_memory[my_idx + 1] = (((c2 & 0xf0) >> 2) | ((c2 & 0xf0) >> 6)) << 2;
      self.shared_memory[my_idx + 2] = (((c2 & 0x0f) >> 2) | ((c2 & 0x0f) << 2)) << 2;

      num_colors += 1;

      if num_colors == 16 {
        num_colors = 0;
        num_palettes += 1;
      }

      my_idx += 3;
      i += 2;
    }

    self.shared_memory[idx] = num_palettes;

    my_idx - idx
  }

  fn disassemble_script(&mut self, script_id: u8, idx: usize) -> usize {
    let script = self.resources_manager.get_file(script_id);
    let script_len = script.len() as u16;
    let mut pc: u16 = 0;
    let mut num_entries: u16 = 0;
    let mut my_idx = idx;

    my_idx += 2; // the first 2 bytes are the num of entries in the array

    while pc < script_len {
      let opcode_value = script[pc as usize];

      write_u16(&mut self.shared_memory, my_idx, pc);

      my_idx += 2;
      pc += 1;

      let opcode = self.virtual_machine.opcodes.get(opcode_value);
      let asm_code = (opcode.get_asm_code)(pc, script);
      let asm_code_as_bytes = asm_code.as_bytes();

      self.shared_memory[my_idx] = asm_code_as_bytes.len() as u8;
      my_idx += 1;

      self.shared_memory[my_idx..my_idx + asm_code_as_bytes.len()].copy_from_slice(asm_code_as_bytes);
      my_idx += asm_code_as_bytes.len();

      pc += ((opcode.len)(pc, script) - 1) as u16;
      num_entries += 1;
    }

    write_u16(&mut self.shared_memory, idx, num_entries);

    my_idx - idx
  }
}