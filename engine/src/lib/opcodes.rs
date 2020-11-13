use crate::resources_manager::ResourcesManager;
use crate::virtual_machine::{VirtualMachine, ScriptRegs};
use crate::video::Video;
use crate::defines::{INACTIVE_THREAD, NUM_THREADS, BASE_PART_ID};
use crate::utils::{read_u8, read_u16, read_i16};

pub enum ActionRequest {
  YieldThread   = 1,
  Blit          = 2,
  LoadPart      = 3,
  PlaySound     = 4
}

pub struct Opcode {
  pub len: fn(pc: u16, mem: &[u8]) -> u8,
  pub get_asm_code: fn(pc: u16, mem: &[u8]) -> String,
  pub exec: fn(vm: &mut VirtualMachine, resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], poly_buffer_1: &[u8], poly_buffer_2: &[u8]) -> u32
}

pub struct Opcodes {
  opcodes: Vec<Opcode>
}

fn build_action_request(action: ActionRequest, param: u8) -> u32 {
  (action as u32) << 24 | (param as u32) << 16
}

fn build_play_sound_action_request(snd_id: u8, freq: u8, vol: u8, channel: u8) -> u32 {
  (ActionRequest::PlaySound as u32) << 24 | (snd_id as u32) << 16 | (((channel as u16) << 6 | freq as u16) as u32) << 8 | vol as u32
}

impl Opcodes {
  pub fn new() -> Opcodes {
    let mut opcodes = vec![
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("MOV r[{:02X}], {:04X}", read_u8(script, pc), read_i16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let dst_reg_id = read_u8(script, pc) as usize;
          vm.registers[dst_reg_id] = read_i16(script, pc + 1);
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("MOV r[{:02X}], r[{:02X}]", read_u8(script, pc), read_u8(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let dst_reg_id = read_u8(script, pc) as usize;
          let src_reg_id = read_u8(script, pc + 1) as usize;
          vm.registers[dst_reg_id] = vm.registers[src_reg_id];
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("ADD r[{:02X}], r[{:02X}]", read_u8(script, pc), read_u8(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let dst_reg_id = read_u8(script, pc) as usize;
          let src_reg_id = read_u8(script, pc + 1) as usize;
          vm.registers[dst_reg_id] = (vm.registers[dst_reg_id] as i32 + vm.registers[src_reg_id] as i32) as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("ADD r[{:02X}], {:04X}", read_u8(script, pc), read_i16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let reg_id = read_u8(script, pc);
          vm.registers[reg_id as usize] = (vm.registers[reg_id as usize] as i32 + read_i16(script, pc + 1) as i32) as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("CALL {:04X}", read_u16(script, pc)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let addr = read_u16(script, pc);

          vm.stack_push(pc + 2);
          vm.threads[thread_id as usize].pc = addr;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 1 },
        get_asm_code: |_pc: u16, _script: &[u8]| { format!("RET") },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, _script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let addr = vm.stack_pop();
          vm.threads[thread_id as usize].pc = addr;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 1 },
        get_asm_code: |_pc: u16, _script: &[u8]| { format!("YIELD") },
        exec: |_vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, _thread_id: u8, _script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          build_action_request(ActionRequest::YieldThread, 0)
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("JMP {:04X}", read_u16(script, pc)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let addr = read_u16(script, pc);
          vm.threads[thread_id as usize].pc = addr;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SETVEC {:02X}, {:04X}", read_u8(script, pc), read_u16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let target_thread_id = read_u8(script, pc);
          let addr = read_u16(script, pc + 1);
          vm.threads[target_thread_id as usize].next_pc = addr;

          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("JNZ r[{:02X}], {:04X}", read_u8(script, pc), read_u16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let reg_id = read_u8(script, pc);

          vm.registers[reg_id as usize] -= 1;

          if vm.registers[reg_id as usize] != 0 {
            vm.threads[thread_id as usize].pc = read_u16(script, pc + 1);
          }
          0
        }
      },
      Opcode {
        len: |pc: u16, script: &[u8]| {
          return if read_u8(script, pc) & 0x40 != 0 { 7 } else { 6 };
        },
        get_asm_code: |pc: u16, script: &[u8]| {
          let mut my_pc = pc;
          let param_type = read_u8(script, my_pc);
          my_pc += 1;
          let param2 = format!("r[{:02X}]", read_u8(script, my_pc));
          my_pc += 1;
          let param1: String;

          if param_type & 0x80 != 0 {
            param1 = format!("r[{:02X}]", read_u8(script, my_pc));
            my_pc += 1;
          } else if param_type & 0x40 != 0 {
            param1 = format!("{:04X}", read_u16(script, my_pc));
            my_pc += 2;
          } else {
            param1 = format!("{:02X}", read_u8(script, my_pc));
            my_pc += 1;
          }

          let jmp_types = ["CJZ", "CJNZ", "CJG", "CJGE", "CJL", "CJLE"];
          format!("{:} {:}, {:}, {:04X}", jmp_types[(param_type & 7) as usize], param2, param1, read_u16(script, my_pc))
        },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let mut pc = vm.threads[thread_id as usize].pc;
          let param_type = read_u8(script, pc);
          pc += 1;
          let param2 = vm.registers[read_u8(script, pc) as usize];
          pc += 1;
          let param1: i16;

          if param_type & 0x80 != 0 {
            param1 = vm.registers[read_u8(script, pc) as usize] as i16;
            pc += 1;
          } else if param_type & 0x40 != 0 {
            param1 = read_i16(script, pc);
            pc += 2;
          } else {
            param1 = read_u8(script, pc) as i16;
            pc += 1;
          }

          let mut condition_passed = false;

          match param_type & 0x7 {
            0 => condition_passed = param2 == param1, // JZ
            1 => condition_passed = param2 != param1, // JNZ
            2 => condition_passed = param2 > param1,  // JG
            3 => condition_passed = param2 >= param1, // JGE
            4 => condition_passed = param2 < param1,  // JL
            5 => condition_passed = param2 <= param1, // JLE
            _ => {} // this shound't happen
          }

          if condition_passed {
            let addr = read_u16(script, pc);
            vm.threads[thread_id as usize].pc = addr;
          }

          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SETPAL {:04X}", read_u16(script, pc)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let palette_id = read_u16(script, pc) >> 8;
          video.set_palette(palette_id as u8);

          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| {
          let origin_thread_id = read_u8(script, pc);
          let target_thread_id = read_u8(script, pc + 1);
          let action_id = read_u8(script, pc + 2);

          let action = match action_id {
            2 => "KILL",
            1 => "YIELD",
            _ => "NONE"
          };

          format!("RESET {:02X}, {:02X}, {:}", origin_thread_id, target_thread_id, action)
        },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let thread_id = read_u8(script, pc);
          let mut i = read_u8(script, pc + 1);

          i &= (NUM_THREADS - 1) as u8;
          let n = i - thread_id + 1;
          let action = read_u8(script, pc + 2);

          if action == 2 {
            for t in thread_id..(thread_id + n) {
              vm.threads[t as usize].next_pc = INACTIVE_THREAD - 1;
            }
          } else if action < 2 {
            for t in thread_id..(thread_id + n) {
              vm.threads[t as usize].next_active = if action == 0 { false } else { true };
            }
          }
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 2 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SETVIDPAG {:02X}", read_u8(script, pc)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          video.set_backbuffer_page(read_u8(script, pc));
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("FILLVIDPAG {:02X}, {:02X}", read_u8(script, pc), read_u8(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          video.fill_page(read_u8(script, pc), read_u8(script, pc + 1));
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("CPVIDPAG {:02X}, {:02X}", read_u8(script, pc), read_u8(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          video.copy_page(read_u8(script, pc), read_u8(script, pc + 1), vm.registers[ScriptRegs::ScrollY as usize]);
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 2 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("BLIT {:02X}", read_u8(script, pc)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let page_id = read_u8(script, pc);

          vm.registers[0xf7] = 0;
          video.blit(page_id);

          build_action_request(ActionRequest::Blit, (vm.registers[ScriptRegs::PauseSlices as usize] * 20) as u8)
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 1 },
        get_asm_code: |_pc: u16, _script: &[u8]| { format!("KILL") },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, _script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          vm.threads[thread_id as usize].pc = INACTIVE_THREAD;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 6 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("DRAWSTR {:04X}, {:02X}, {:02X}, {:02X}", read_u16(script, pc), read_u8(script, pc + 2), read_u8(script, pc + 3), read_u8(script, pc + 4)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          video.draw_string(read_u16(script, pc), read_u8(script, pc + 2) as i16, read_u8(script, pc + 3) as i16, read_u8(script, pc + 4));
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SUB r[{:02X}], r[{:02X}]", read_u8(script, pc), read_u8(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          vm.registers[read_u8(script, pc) as usize] -= vm.registers[read_u8(script, pc + 1) as usize] as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("AND r[{:02X}], {:04X}", read_u8(script, pc), read_u16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          let reg_id = read_u8(script, pc) as usize;
          let value = read_u16(script, pc + 1);
          vm.registers[reg_id] = (vm.registers[reg_id] as u16 & value) as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("OR r[{:02X}], {:04X}", read_u8(script, pc), read_u16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          vm.registers[read_u8(script, pc) as usize] = (vm.registers[read_u8(script, pc) as usize] as u16 | read_u16(script, pc + 1)) as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SHL r[{:02X}], {:04X}", read_u8(script, pc), read_u16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          vm.registers[read_u8(script, pc) as usize] = ((vm.registers[read_u8(script, pc) as usize] as u16) << read_u16(script, pc + 1)) as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 4 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SHR r[{:02X}], {:04X}", read_u8(script, pc), read_u16(script, pc + 1)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          vm.registers[read_u8(script, pc) as usize] = (vm.registers[read_u8(script, pc) as usize] as u16 >> read_u16(script, pc + 1)) as i16;
          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 6 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("SND {:04X}, {:02X}, {:02X}, {:02X}", read_u16(script, pc), read_u8(script, pc + 2), read_u8(script, pc + 3), read_u8(script, pc + 4)) },
        exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let pc = vm.threads[thread_id as usize].pc;
          build_play_sound_action_request(read_u16(script, pc) as u8, read_u8(script, pc + 2), read_u8(script, pc + 3), read_u8(script, pc + 4))
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 3 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("LDRES {:04X}", read_u16(script, pc)) },
        exec: |vm: &mut VirtualMachine, resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          let resource_id = read_u16(script, vm.threads[thread_id as usize].pc);

          // all the resources are already loaded in memory, so here we check:
          // 1. if it's trying to load a game part
          if resource_id > 0xff {
            let part = (resource_id - BASE_PART_ID) as u8;
            vm.set_next_part_to_load(part);

            return build_action_request(ActionRequest::LoadPart, part);
          }

          // 2. if it's trying to load a bitmap. In this case, the bitmap is copied to page 0
          if resources_manager.get_file_type(resource_id as u8) == 0x2 {
            video.draw_bitmap(resources_manager.get_file(resource_id as u8));
          }

          0
        }
      },
      Opcode {
        len: |_pc: u16, _script: &[u8]| { 6 },
        get_asm_code: |pc: u16, script: &[u8]| { format!("MUSIC {:04X}, {:04X}, {:02X}", read_u16(script, pc), read_u16(script, pc + 2), read_u8(script, pc + 4)) },
        exec: |_vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, _video: &mut Video, _thread_id: u8, _script: &[u8], _poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
          // TODO: to implement
          0
        }
      }
    ];

    opcodes.push(Opcode {
      len: |_pc: u16, _script: &[u8]| { 4 },
      get_asm_code: |pc: u16, script: &[u8]| {
        let opcode = read_u8(script, pc - 1);

        let mut offset = (opcode as u16) << 8;
        offset |= read_u8(script, pc) as u16;
        offset = ((offset as u32) * 2) as u16;

        let mut x = read_u8(script, pc + 1) as i16;
        let mut y = read_u8(script, pc + 2) as i16;
        let h = y - 199;

        if h > 0 {
          y = 199;
          x += h;
        }

        format!("DRAWPOLY1 {:04X}, {:02X}, {:02X}, {:02X}", offset, x, y, 0x40)
      },
      exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], poly_buffer_1: &[u8], _poly_buffer_2: &[u8]| -> u32 {
        let mut pc = vm.threads[thread_id as usize].pc;
        let opcode = read_u8(script, pc - 1);

        let mut offset = (opcode as u16) << 8;
        offset |= read_u8(script, pc) as u16;
        offset = ((offset as u32) * 2) as u16;
        pc += 1;

        let mut x = read_u8(script, pc) as i16;
        let mut y = read_u8(script, pc + 1) as i16;
        let h = y - 199;

        if h > 0 {
          y = 199;
          x += h;
        }

        video.draw_poly(poly_buffer_1, offset, x, y, 0x40);

        0
      }
    });

    opcodes.push(Opcode {
      len: |pc: u16, script: &[u8]| {
        // this opcode modifies the pc, so the len is calculated taking this into account
        let mut len = 6;
        let opcode = read_u8(script, pc - 1);

        if opcode & 0x20 == 0 && opcode & 0x10 == 0 {
          len += 1;
        }

        if opcode & 0x8 == 0 && opcode & 0x4 == 0 {
          len += 1;
        }

        if opcode & 0x2 == 0 {
          if opcode & 0x1 == 0 {
            len -= 1;
          }
        } else if opcode & 0x1 != 0 {
          len -= 1;
        }

        len
      },
      get_asm_code: |pc: u16, script: &[u8]| {
        let mut my_pc = pc;
        let opcode = read_u8(script, my_pc - 1);
        let offset = read_u16(script, my_pc) * 2;
        my_pc += 2;

        let mut x = read_u8(script, my_pc) as i16;
        my_pc += 1;

        let mut x_unknown = false;
        let mut y_unknown = false;

        if opcode & 0x20 == 0 {
          if opcode & 0x10 == 0 {
            x = (x << 8) | read_u8(script, my_pc) as i16;
            my_pc += 1;
          } else {
            x_unknown = true;
          }
        } else if opcode & 0x10 != 0 {
          x += 0x100;
        }

        let mut y = read_u8(script, my_pc) as i16;
        my_pc += 1;

        if opcode & 0x8 == 0 {
          if opcode & 0x4 == 0 {
            y = (y << 8) | read_u8(script, my_pc) as i16;
            my_pc += 1;
          } else {
            y_unknown = true;
          }
        }

        let mut poly = 1;
        let mut zoom = read_u8(script, my_pc) as i16;
        let mut zoom_unknown = false;

        if opcode & 0x2 == 0 {
          if opcode & 0x1 == 0 {
            zoom = 0x40;
          } else {
            zoom_unknown = true;
          }
        } else if opcode & 0x1 != 0 {
          zoom = 0x40;
          poly = 2;
        }

        let final_x = if x_unknown { "??".to_string() } else { format!("{:02X}", x) };
        let final_y = if y_unknown { "??".to_string() } else { format!("{:02X}", y) };
        let final_zoom = if zoom_unknown { "??".to_string() } else { format!("{:02X}", zoom) };

        format!("DRAWPOLY{:} {:04X}, {:}, {:}, {:}", poly, offset, final_x, final_y, final_zoom)
      },
      exec: |vm: &mut VirtualMachine, _resources_manager: &ResourcesManager, video: &mut Video, thread_id: u8, script: &[u8], poly_buffer_1: &[u8], poly_buffer_2: &[u8]| -> u32 {
        let mut pc = vm.threads[thread_id as usize].pc;
        let opcode = read_u8(script, pc - 1);
        let offset = read_u16(script, pc) * 2;
        pc += 2;

        let mut x = read_u8(script, pc) as i16;
        pc += 1;

        if opcode & 0x20 == 0 {
          if opcode & 0x10 == 0 {
            x = (x << 8) | read_u8(script, pc) as i16;
            pc += 1;
          } else {
            x = vm.registers[x as usize];
          }
        } else if opcode & 0x10 != 0 {
          x += 0x100;
        }

        let mut y = read_u8(script, pc) as i16;
        pc += 1;

        if opcode & 0x8 == 0 {
          if opcode & 0x4 == 0 {
            y = (y << 8) | read_u8(script, pc) as i16;
            pc += 1;
          } else {
            y = vm.registers[y as usize];
          }
        }

        let mut poly_buffer = poly_buffer_1;
        let mut zoom = read_u8(script, pc) as i16;

        if opcode & 0x2 == 0 {
          if opcode & 0x1 == 0 {
            zoom = 0x40;
          } else {
            zoom = vm.registers[zoom as usize];
          }
        } else if opcode & 0x1 != 0 {
          zoom = 0x40;
          poly_buffer = poly_buffer_2;
        }

        video.draw_poly(poly_buffer, offset, x, y, zoom);

        0
      }
    });

    Opcodes {
      opcodes: opcodes
    }
  }

  pub fn get(&self, opcode: u8) -> &Opcode {
    // is it a draw_poly opcode?
    if opcode & 0x80 != 0 {
      return &self.opcodes[self.opcodes.len() - 2];
    }

    if opcode & 0x40 != 0 {
      return &self.opcodes[self.opcodes.len() - 1];
    }

    &self.opcodes[opcode as usize]
  }
}