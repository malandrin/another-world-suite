use std::collections::HashMap;

struct UnpackContext<'a> {
  size: u16,
  crc: u32,
  chk: u32,
  data_size: u32,
  index: u32,
  output_index: u32,
  packed_content: &'a[u8],
  output_content: &'a mut[u8],
  u32buffer: [u8; 4]
}

impl<'a> UnpackContext<'a> {
  pub fn read_u32(&mut self) -> u32 {
    self.u32buffer.copy_from_slice(&self.packed_content[self.index as usize..(self.index + 4) as usize]);

    if self.index > 0 {
      self.index -= 4;
    }

    u32::from_be_bytes(self.u32buffer)
  }
}

pub enum ResourceType {
  Bitmap = 2,
  Palette = 3,
  Script = 4
}

pub struct FileEntry {
  pub ftype: u8,
  pub content: Vec<u8>
}

pub struct ResourcesManager {
  pub files: Vec<FileEntry>
}

impl ResourcesManager {
  pub fn new() -> ResourcesManager {

    ResourcesManager {
      files: Vec::new()
    }
  }

  pub fn init(&mut self, game_data: &[u8]) {
    let mut files_dict: HashMap<String, &[u8]> = HashMap::new();
    let mut buffer_len: [u8; 4] = [0; 4];
    buffer_len.copy_from_slice(&game_data[0..4]);
    let mut file_len = u32::from_le_bytes(buffer_len);

    files_dict.insert("memlist.bin".to_string(), &game_data[4 as usize..(4 + file_len) as usize]);

    let mut content_idx = 4 + file_len;

    for i in 1..14 {
      let filename = format!("bank0{:x}", i);
      buffer_len.copy_from_slice(&game_data[content_idx as usize..(content_idx + 4) as usize]);
      file_len = u32::from_le_bytes(buffer_len);
      content_idx += 4;
      files_dict.insert(filename, &game_data[content_idx as usize..(content_idx + file_len) as usize]);
      content_idx += file_len;
    }

    // parse memlist and load all the files in memory. Memlist contains the information about the files used by the game, as the resource type, the size, in which bank the content of the file is,
    // in which position starts, etc.
    let memlist = files_dict.get("memlist.bin").unwrap();
    let mut idx = 0;
    let mut u32buffer: [u8; 4] = [0; 4];
    let mut u16buffer: [u8; 2] = [0; 2];

    loop {
      let state = memlist[idx];
      idx += 1;

      if state == 0xff { // end of memlist
        break;
      }

      let ftype = memlist[idx];
      idx += 1;

      // unknow
      idx += 4;

      // rankNum
      idx += 1;

      let bank_id = memlist[idx];
      idx += 1;

      u32buffer.copy_from_slice(&memlist[idx..idx + 4]);
      let bank_offset = u32::from_be_bytes(u32buffer);
      idx += 4;

      // unknow
      idx += 2;

      u16buffer.copy_from_slice(&memlist[idx..idx + 2]);
      let packed_size = u16::from_be_bytes(u16buffer);
      idx += 2;

      // unknow
      idx += 2;

      u16buffer.copy_from_slice(&memlist[idx..idx + 2]);
      let size = u16::from_be_bytes(u16buffer);
      idx += 2;

      if ftype == ResourceType::Bitmap as u8 { // bitmap
        self.files.push(FileEntry {
          ftype: ftype,
          content: self.create_bitmap(&self.load_file(&files_dict, bank_id, bank_offset, size, packed_size))
        })
      } else {
        self.files.push(FileEntry {
          ftype: ftype,
          content: self.load_file(&files_dict, bank_id, bank_offset, size, packed_size)
        });
      }
    }
  }

  pub fn get_file(&self, file_id: u8) -> &[u8] {
    &self.files[file_id as usize].content
  }

  pub fn get_file_type(&self, file_id: u8) -> u8 {
    self.files[file_id as usize].ftype
  }

  fn create_bitmap(&self, content: &Vec<u8>) -> Vec<u8> {
    let mut bitmap = Vec::new();
    let mut src_idx = 0;

    while src_idx < 8000 {
      let mut p: [u8; 4] = [content[src_idx + 8000 * 3], content[src_idx + 8000 * 2], content[src_idx + 8000], content[src_idx]];

      for _ in 0..4 {
        let mut acc = 0;

        for i in 0..8 {
          acc <<= 1;
          acc |= if (p[i & 3] & 0x80) != 0 { 1 } else { 0 };
          p[i & 3] <<= 1;
        }

        bitmap.push(acc >> 4);
        bitmap.push(acc & 0xf);
      }

      src_idx += 1;
    }

    bitmap
  }

  fn load_file(&self, files_dict: &HashMap<String, &[u8]>, bank_id: u8, bank_offset: u32, size: u16, packed_size: u16) -> Vec<u8> {
    let mut content = vec!(0; size as usize);

    if size > 0 {
      let bank = files_dict.get(&format!("bank0{:x}", bank_id)).unwrap();

      if size == packed_size {
        let mem = &bank[bank_offset as usize..(bank_offset + size as u32) as usize];
        content.clone_from_slice(mem);
      } else {
        // unpack code taken from https://github.com/fabiensanglard/Another-World-Bytecode-Interpreter/blob/master/src/bank.cpp
        let mut unpack_context = UnpackContext{
          size: 0,
          crc: 0,
          chk: 0,
          data_size: size as u32,
          index: (packed_size - 4) as u32,
          output_index: (size - 1) as u32,
          packed_content: &bank[bank_offset as usize..(bank_offset + packed_size as u32) as usize],
          output_content: &mut content,
          u32buffer: [0; 4]
        };

        unpack_context.data_size = unpack_context.read_u32();
        unpack_context.crc = unpack_context.read_u32();
        unpack_context.chk = unpack_context.read_u32();
        unpack_context.crc = unpack_context.crc ^ unpack_context.chk;

        while unpack_context.data_size > 0 {
          if !self.next_chunk(&mut unpack_context) {
            unpack_context.size = 1;
            if !self.next_chunk(&mut unpack_context) {
              self.dec_unk_1(&mut unpack_context, 3, 0);
            } else {
              self.dec_unk_2(&mut unpack_context, 8);
            }
          } else {
            let c = self.get_code(&mut unpack_context, 2);
            if c == 3 {
              self.dec_unk_1(&mut unpack_context, 8, 8);
            }
            else {
              if c < 2 {
                unpack_context.size = c + 2;
                self.dec_unk_2(&mut unpack_context, (c + 9) as u8);
              } else {
                unpack_context.size = self.get_code(&mut unpack_context, 8);
                self.dec_unk_2(&mut unpack_context, 12);
              }
            }
          }
        }
      }
    }

    content
  }

  fn dec_unk_1(&self, unpack_context: &mut UnpackContext, num_chunks: u8, add_count: u8) {
    let mut count = self.get_code(unpack_context, num_chunks) + add_count as u16 + 1;
    unpack_context.data_size -= count as u32;

    while count > 0 {
      let val = self.get_code(unpack_context, 8) as u8;
      unpack_context.output_content[unpack_context.output_index as usize] = val;

      if unpack_context.output_index > 0 {
        unpack_context.output_index -= 1;
      }

      count -= 1;
    }
  }

  fn dec_unk_2(&self, unpack_context: &mut UnpackContext, num_chunks: u8) {
    let i = self.get_code(unpack_context, num_chunks) as u32;
    let mut count = unpack_context.size + 1;

    unpack_context.data_size -= count as u32;

    while count > 0 {
      let val = unpack_context.output_content[(unpack_context.output_index + i) as usize];
      unpack_context.output_content[unpack_context.output_index as usize] = val;

      if unpack_context.output_index > 0 {
        unpack_context.output_index -= 1;
      }

      count -= 1;
    }
  }

  fn get_code(&self, unpack_context: &mut UnpackContext, num_chunks: u8) -> u16 {
    let mut c: u16 = 0;
    let mut n = num_chunks;

    while n > 0 {
      c = c << 1;
      if self.next_chunk(unpack_context) {
        c = c | 1;
      }
      n -= 1
    }

    c
  }

  fn next_chunk(&self, unpack_context: &mut UnpackContext) -> bool {
    let mut cf = self.rcr(unpack_context, false);

    if unpack_context.chk == 0 {
      unpack_context.chk = unpack_context.read_u32();
      unpack_context.crc = unpack_context.crc ^ unpack_context.chk;
      cf = self.rcr(unpack_context, true);
    }

    cf
  }

  fn rcr(&self, unpack_context: &mut UnpackContext, cf: bool) -> bool {
    let rcf = (unpack_context.chk & 1) != 0;

    unpack_context.chk = unpack_context.chk >> 1;
    if cf {
      unpack_context.chk |= 0x80000000;
    }

    rcf
  }
}