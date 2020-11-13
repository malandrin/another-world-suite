use crate::resources_manager::ResourcesManager;
use crate::opcodes::{Opcodes, ActionRequest};
use crate::video::Video;
use crate::defines::*;

enum Keys {
  Up      = 1 << 0,
  Right   = 1 << 1,
  Down    = 1 << 2,
  Left    = 1 << 3,
  Action  = 1 << 4
}

pub enum ScriptRegs {
  RandomSeed        = 0x3c,
  LastKeyChar       = 0xda,
  HeroPosUpDown     = 0xe5,
  MusMark           = 0xf4,
  ScrollY           = 0xf9,
  HeroAction        = 0xfa,
  HeroPosJumpDown   = 0xfb,
  HeroPosLeftRight  = 0xfc,
  HeroPosMask       = 0xfd,
  HeroActionPosMask = 0xfe,
  PauseSlices       = 0xff
}

pub struct Thread {
  pub pc: u16,
  pub next_pc: u16,
  pub active: bool,
  pub next_active: bool
}

impl Thread {
  pub fn new() -> Thread {
    Thread {
      pc: 0,
      next_pc: 0,
      active: false,
      next_active: false
    }
  }
}

pub struct VirtualMachine {
  pub registers: Vec<i16>,
  pub opcodes: Opcodes,
  pub threads: Vec<Thread>,
  pub active_thread: u8,
  pub script_file_id: u8,
  pub palette_file_id: u8,
  pub next_part_id: u8,
  stack: Vec<u16>,
  polys1_file_id: u8,
  polys2_file_id: u8,
  direction_keys_enabled: u8,
  action_key_enabled: bool
}

impl VirtualMachine {
  pub fn new() -> VirtualMachine {
    let mut threads = Vec::with_capacity(20);

    for _ in 0..NUM_THREADS {
      threads.push(Thread::new());
    }

    VirtualMachine {
      registers: vec![0; NUM_REGISTERS],
      threads: threads,
      opcodes: Opcodes::new(),
      active_thread: 0,
      stack: Vec::with_capacity(NUM_THREADS),
      script_file_id: 0,
      polys1_file_id: 0,
      polys2_file_id: 0,
      palette_file_id: 0,
      next_part_id: 0,
      direction_keys_enabled: 0,
      action_key_enabled: false
    }
  }

  pub fn init(&mut self) {
    self.registers[ScriptRegs::RandomSeed as usize] = 0; // not very a random number
  }

  pub fn restart_level(&mut self, level: u8) {
    for i in 0..64 {
      self.registers[i] = 0;
    }

    let mut part = 0;

    match level {
      0 => { // the intro
        for i in 0..LEVEL_00_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_00_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_00_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 1;
      },
      1 => {
        for i in 0..LEVEL_01_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_01_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_01_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 2;
      },
      2 => {
        for i in 0..LEVEL_02_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_02_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_02_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 3;
      },
      3 => {
        for i in 0..LEVEL_03_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_03_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_03_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      4 => {
        for i in 0..LEVEL_04_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_04_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_04_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      5 => {
        for i in 0..LEVEL_05_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_05_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_05_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      6 => {
        for i in 0..LEVEL_06_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_06_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_06_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      7 => {
        for i in 0..LEVEL_07_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_07_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_07_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      8 => {
        for i in 0..LEVEL_08_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_08_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_08_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      9 => {
        for i in 0..LEVEL_09_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_09_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_09_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      10 => {
        for i in 0..LEVEL_10_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_10_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_10_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      11 => {
        for i in 0..LEVEL_11_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_11_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_11_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 4;
      },
      12 => {
        for i in 0..LEVEL_12_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_12_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_12_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 6;
      },
      13 => {
        for i in 0..LEVEL_13_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_13_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_13_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 6;
      },
      14 => {
        for i in 0..LEVEL_14_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_14_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_14_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 5;
      },
      15 => {
        for i in 0..LEVEL_15_INITIAL_REGISTERS_VALUES.len() {
          self.registers[LEVEL_15_INITIAL_REGISTERS_VALUES[i][0] as usize] = LEVEL_15_INITIAL_REGISTERS_VALUES[i][1];
        }
        part = 6;
      },
      _ => {}
    }

    self.load_part(part);
  }

  pub fn set_next_part_to_load(&mut self, part: u8) {
    self.next_part_id = part;
  }

  pub fn on_key_down(&mut self, key: u8) {
    if key >= Keys::Up as u8 && key <= Keys::Left as u8 {
      self.direction_keys_enabled |= key;
    } else if key == Keys::Action as u8 {
      self.action_key_enabled = true;
    }
  }

  pub fn on_key_up(&mut self, key: u8) {
    if key >= Keys::Up as u8 && key <= Keys::Left as u8 {
      self.direction_keys_enabled &= !key;
    } else if key == Keys::Action as u8 {
      self.action_key_enabled = false;
    }
  }

  pub fn step(&mut self, video: &mut Video, resources_manager: &ResourcesManager) -> u32 {
    if self.next_part_id != 0 {
      self.load_part(self.next_part_id);
      self.next_part_id = 0;
    }

    let tidx = self.active_thread as usize;

    let action_requested = self.thread_step(
      resources_manager,
      video,
      self.active_thread,
      resources_manager.get_file(self.script_file_id),
      resources_manager.get_file(self.polys1_file_id),
      resources_manager.get_file(self.polys2_file_id)
    );

    let action = (action_requested >> 24) as u8;

    if action == ActionRequest::YieldThread as u8 || self.threads[tidx].pc == INACTIVE_THREAD {
      let mut idx = ((self.active_thread + 1) as usize) % NUM_THREADS;

      loop {
        // after loop over all the threads, the pc is set based on the next_pc variable
        if idx == 0 {
          self.process_input();

          for i in 0..NUM_THREADS {
            self.threads[i].active = self.threads[i].next_active;

            if self.threads[i].next_pc != INACTIVE_THREAD {
              if self.threads[i].next_pc == INACTIVE_THREAD - 1 {
                self.threads[i].pc = INACTIVE_THREAD;
              } else {
                self.threads[i].pc = self.threads[i].next_pc;
              }
              self.threads[i].next_pc = INACTIVE_THREAD;
            }
          }
        }

        if self.threads[idx].pc != INACTIVE_THREAD && !self.threads[idx].active {
          self.active_thread = idx as u8;
          break;
        }

        idx = (idx + 1) % NUM_THREADS;
      }
    }

    // only return the action requested that should be managed by the host system
    if action >= ActionRequest::Blit as u8 {
      return action_requested;
    }

    0
  }

  pub fn stack_push(&mut self, value: u16) {
    self.stack.push(value);
  }

  pub fn stack_pop(&mut self) -> u16 {
    self.stack.pop().unwrap()
  }

  pub fn get_current_pc(&self) -> u16 {
    self.threads[self.active_thread as usize].pc
  }

  fn thread_step(&mut self, resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], poly1: &[u8], poly2: &[u8]) -> u32 {
    let tidx = thread_id as usize;
    let pc = self.threads[tidx].pc;

    if pc == INACTIVE_THREAD {
      return 0;
    }

    self.threads[tidx].pc += 1;

    let opcode_value = script[pc as usize];
    let opcode = self.opcodes.get(opcode_value);
    let opcode_len = ((opcode.len)(self.threads[tidx].pc, script) - 1) as u16;

    let action_requested = (opcode.exec)(
      self,
      resources_manager,
      video,
      thread_id,
      script,
      poly1,
      poly2
    );

    if pc + 1 == self.threads[tidx].pc { // the instruction is not a call, ret or jmp
      self.threads[tidx].pc += opcode_len;
    }

    action_requested
  }

  fn load_part(&mut self, part: u8) {
    match part {
      0 => { // protection screen
        self.script_file_id = 0x15;
        self.polys1_file_id = 0x16;
        self.polys2_file_id = 0x0;
        self.palette_file_id = 0x14;
      },
      1 => { // introduction
        self.script_file_id = 0x18;
        self.polys1_file_id = 0x19;
        self.polys2_file_id = 0x0;
        self.palette_file_id = 0x17;
      },
      2 => {
        self.script_file_id = 0x1b;
        self.polys1_file_id = 0x1c;
        self.polys2_file_id = 0x11;
        self.palette_file_id = 0x1a;
      },
      3 => {
        self.script_file_id = 0x1e;
        self.polys1_file_id = 0x1f;
        self.polys2_file_id = 0x11;
        self.palette_file_id = 0x1d;
      },
      4 => {
        self.script_file_id = 0x21;
        self.polys1_file_id = 0x22;
        self.polys2_file_id = 0x11;
        self.palette_file_id = 0x20;
      },
      5 => {
        self.script_file_id = 0x24;
        self.polys1_file_id = 0x25;
        self.polys2_file_id = 0x0;
        self.palette_file_id = 0x23;
      },
      6 => {
        self.script_file_id = 0x27;
        self.polys1_file_id = 0x28;
        self.polys2_file_id = 0x11;
        self.palette_file_id = 0x26;
      },
      _ => panic!("invalid game part")
    }

    self.registers[0xe4] = 0x14;

    for i in 0..NUM_THREADS {
      self.threads[i].pc = INACTIVE_THREAD;
      self.threads[i].next_pc = INACTIVE_THREAD;
      self.threads[i].active = false;
      self.threads[i].next_active = false;
    }

    self.threads[0].pc = 0;
    self.active_thread = 0;
  }

  fn process_input(&mut self) {
    let mut left_right: i16 = 0;
    let mut up_down: i16 = 0;
    let mut mask: i16 = 0;

    if (self.direction_keys_enabled & Keys::Right as u8) != 0 {
      left_right = 1;
      mask |= 1;
    }

    if (self.direction_keys_enabled & Keys::Left as u8) != 0 {
      left_right = -1;
      mask |= 2;
    }

    if (self.direction_keys_enabled & Keys::Down as u8) != 0 {
      up_down = 1;
      mask |= 4;
    }

    if (self.direction_keys_enabled & Keys::Up as u8) != 0 {
      up_down = -1;
      mask |= 8;
    }

    self.registers[ScriptRegs::HeroPosLeftRight as usize] = left_right;
    self.registers[ScriptRegs::HeroPosUpDown as usize] = up_down;
    self.registers[ScriptRegs::HeroPosJumpDown as usize] = up_down;
    self.registers[ScriptRegs::HeroPosMask as usize] = mask;

    if self.action_key_enabled {
      self.registers[ScriptRegs::HeroAction as usize] = 1;
      self.registers[ScriptRegs::HeroActionPosMask as usize] = mask | 0x80;
    } else {
      self.registers[ScriptRegs::HeroAction as usize] = 0;
      self.registers[ScriptRegs::HeroActionPosMask as usize] = mask;
    }
  }
}
