use std::collections::HashMap;
use crate::defines::{FRAME_BUFFER_WIDTH, FRAME_BUFFER_HEIGHT, INVALID_PALETTE};
use crate::game_strings::init_game_strings;
use crate::font::FONT;
use crate::poly::{Poly, draw_poly_to_buffer};

const NUM_PAGES: usize = 4;

pub struct Video {
  pages: [Vec<u8>; NUM_PAGES],
  frontbuffer_page_idx: usize,
  backbuffer_page_idx: usize,
  background_builder_page_idx: usize,
  palette_id: u8,
  next_palette_id: u8,
  game_strings: HashMap<u16, &'static str>
}

impl Video {
  pub fn new() -> Video {
    let size = (FRAME_BUFFER_WIDTH * FRAME_BUFFER_HEIGHT) as usize;
    let mut game_strings = HashMap::new();

    init_game_strings(&mut game_strings);

    Video {
      pages: [vec![0; size], vec![0; size], vec![0; size], vec![0; size]],
      frontbuffer_page_idx: 2,
      backbuffer_page_idx: 2,
      background_builder_page_idx: 1,
      palette_id: 0,
      next_palette_id: INVALID_PALETTE,
      game_strings: game_strings
    }
  }

  pub fn get_active_palette_id(&self) -> u8 {
    self.palette_id
  }

  pub fn get_screen_page(&self) -> &[u8] {
    &self.pages[self.backbuffer_page_idx]
  }

  pub fn blit(&mut self, page_id: u8) {
    if page_id == 0xff {
      let tmp = self.frontbuffer_page_idx;
      self.frontbuffer_page_idx = self.background_builder_page_idx;
      self.background_builder_page_idx = tmp;
    } else if page_id != 0xfe {
      self.frontbuffer_page_idx = self.get_page_idx(page_id);
    }

    if self.next_palette_id != INVALID_PALETTE {
      self.palette_id = self.next_palette_id;
      self.next_palette_id = INVALID_PALETTE;
    }
  }

  pub fn fill_page(&mut self, page_id: u8, color_idx: u8) {
    let page_idx = self.get_page_idx(page_id);

    for i in &mut self.pages[page_idx] {
      *i = color_idx;
    }
  }

  pub fn copy_page(&mut self, src_page_id: u8, dst_page_id: u8, vscroll: i16) {
    if src_page_id == dst_page_id {
      return;
    }

    let spage = src_page_id & 0xbf;

    if src_page_id >= 0xfe || ((spage & 0x80) == 0) {
      let s = self.get_page_idx(if src_page_id >= 0xfe { src_page_id } else { spage });
      let d = self.get_page_idx(dst_page_id);

      if s == d {
        return;
      }

      for i in 0..(FRAME_BUFFER_WIDTH * FRAME_BUFFER_HEIGHT) as usize {
        self.pages[d][i] = self.pages[s][i];
      }
    } else {
      let s = self.get_page_idx(spage & 3);
      let d = self.get_page_idx(dst_page_id);

      if vscroll.abs() < FRAME_BUFFER_HEIGHT as i16 {
        let source_y = vscroll.abs();
        let dest_y = if vscroll < 0 { 0 } else { vscroll };
        let height = FRAME_BUFFER_HEIGHT as i16 - source_y;

        for y in 0..height {
          let ix = (source_y + y) as usize * FRAME_BUFFER_WIDTH as usize;
          let iy = (dest_y + y) as usize * FRAME_BUFFER_WIDTH as usize;

          for x in 0..FRAME_BUFFER_WIDTH {
            self.pages[d][ix + x as usize] = self.pages[s][iy + x as usize];
          }
        }
      }
    }
  }

  pub fn set_palette(&mut self, palette_id: u8) {
    self.next_palette_id = palette_id
  }

  pub fn set_backbuffer_page(&mut self, page_id: u8) {
    self.backbuffer_page_idx = self.get_page_idx(page_id);
  }

  pub fn draw_poly(&mut self, poly_buffer: &[u8], offset: u16, x: i16, y: i16, zoom: i16) {
    let mut poly = Poly::new();
    draw_poly_to_buffer(&mut poly, poly_buffer, offset, x, y, zoom, 0xff, &mut self.pages, self.backbuffer_page_idx);
  }

  pub fn draw_bitmap(&mut self, bitmap: &[u8]) {
    self.pages[0].clone_from_slice(bitmap);
  }

  pub fn draw_string(&mut self, string_id: u16, x: i16, y: i16, color_idx: u8) {
    let string = self.game_strings[&string_id];
    let mut wx = x * 8;
    let mut wy = y;

    for c in string.chars() {
      if c == '\n' {
        wx = x * 8;
        wy += 8;
      } else {
        self.draw_char(c, wx, wy, color_idx);
        wx += 8;
      }
    }
  }

  fn draw_char(&mut self, c: char, x: i16, y: i16, color_idx: u8) {
    let char_idx = ((c as u8 - ' ' as u8) as usize) * 8;
    let char_info = &FONT[char_idx .. char_idx + 8];

    for row in 0..8 {
      for col in 0..8 {
        if (char_info[row] & (1 << (7 - col))) != 0 {
          self.pages[self.backbuffer_page_idx][((y + row as i16) as u16 * FRAME_BUFFER_WIDTH + x as u16 + col) as usize] = color_idx;
        }
      }
    }
  }

  fn get_page_idx(&self, page_id: u8) -> usize {
    if page_id <= 3 {
      return page_id as usize
    }

    if page_id == 0xff {
      return self.background_builder_page_idx;
    } else if page_id == 0xfe {
      return self.frontbuffer_page_idx;
    }

    0
  }
}
