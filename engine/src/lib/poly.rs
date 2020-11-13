use std::cmp;
use crate::utils::{read_u16};
use crate::defines::{FRAME_BUFFER_WIDTH, FRAME_BUFFER_HEIGHT};

const MAX_NUM_VERTICES: usize = 50;

pub fn draw_poly_to_buffer(poly: &mut Poly, poly_buffer: &[u8], offset: u16, x: i16, y: i16, zoom: i16, color_idx: u8, output_pages: &mut [Vec<u8>; 4], backbuffer_page_idx: usize) {
  let mut data_index = offset as usize;
  let mut info = poly_buffer[data_index];
  data_index += 1;

  if info >= 0xc0 {
    let mut final_color_idx = color_idx;

    if (final_color_idx & 0x80) != 0 {
      final_color_idx = info & 0x3f;
    }

    // read vertices
    poly.bounding_box_width = (poly_buffer[data_index] as i32 * zoom as i32 / 64) as i16;
    poly.bounding_box_height = (poly_buffer[data_index + 1] as i32 * zoom as i32 / 64) as i16;
    poly.num_vertices = poly_buffer[data_index + 2];

    data_index += 3;

    for i in 0..poly.num_vertices {
      poly.vertices[i as usize] = [(poly_buffer[data_index] as i32 * zoom as i32 / 64) as i16, (poly_buffer[data_index + 1] as i32 * zoom as i32 / 64) as i16];
      data_index += 2;
    }

    poly.draw(x, y, final_color_idx, output_pages, backbuffer_page_idx);
  } else {
    info = info & 0x3f;

    if info == 2 {
      draw_hierarchy_poly_to_buffer(poly, &poly_buffer, data_index as u16, x, y, zoom, output_pages, backbuffer_page_idx);
    }
  }
}

fn draw_hierarchy_poly_to_buffer(poly: &mut Poly, poly_buffer: &[u8], offset: u16, x: i16, y: i16, zoom: i16, output_pages: &mut [Vec<u8>; 4], backbuffer_page_idx: usize) {
  let mut data_index = offset as usize;
  let nx = (x as i32 - poly_buffer[data_index] as i32 * zoom as i32 / 64) as i16;
  let ny = (y as i32 - poly_buffer[data_index + 1] as i32 * zoom as i32 / 64) as i16;
  let num_children = poly_buffer[data_index + 2] + 1;

  data_index += 3;

  for _ in 0..num_children {
    let mut off = read_u16(poly_buffer, data_index as u16);
    let cx = (nx as i32 + poly_buffer[data_index + 2] as i32 * zoom as i32 / 64) as i16;
    let cy = (ny as i32 + poly_buffer[data_index + 3] as i32 * zoom as i32 / 64) as i16;

    data_index += 4;

    let mut final_color_idx = 0xff;
    let bp = off;

    off &= 0x7fff;

    if bp & 0x8000 != 0 {
      final_color_idx = poly_buffer[data_index] & 0x7f;
      data_index += 2;
    }

    draw_poly_to_buffer(poly, poly_buffer, off * 2, cx, cy, zoom, final_color_idx, output_pages, backbuffer_page_idx);
  }
}

pub struct Poly {
  pub bounding_box_width: i16,
  pub bounding_box_height: i16,
  pub num_vertices: u8,
  pub vertices: Vec<[i16; 2]>
}

impl Poly {
  pub fn new() -> Poly {
    Poly {
      bounding_box_width: 0,
      bounding_box_height: 0,
      num_vertices: 0,
      vertices: vec![[0, 0]; MAX_NUM_VERTICES]
    }
  }

  pub fn draw(&self, x: i16, y: i16, color: u8, output_pages: &mut [Vec<u8>; 4], backbuffer_page_idx: usize) {
    if self.bounding_box_width == 0 && self.bounding_box_height == 1 && self.num_vertices == 4 { // it's a point
      if x < 0 || x >= FRAME_BUFFER_WIDTH as i16 || y < 0 || y >= FRAME_BUFFER_HEIGHT as i16 {
        return;
      }

      let offset = ((y as i32 * FRAME_BUFFER_WIDTH as i32) + x as i32) as usize;

      if color == 0x10 {
        output_pages[backbuffer_page_idx][offset] = 0x8;
      } else if color == 0x11 {
        output_pages[backbuffer_page_idx][offset] = output_pages[0][offset];
      } else {
        output_pages[backbuffer_page_idx][offset] = color
      }
    }

    // if some point of the bounding box is outside the screen, the poly is discarded
    let bbwh = self.bounding_box_width / 2;
    let bbhh = self.bounding_box_height / 2;

    let mut x1 = x - bbwh;
    let mut x2 = x + bbwh;
    let y1 = y - bbhh;
    let y2 = y + bbhh;

    if x2 < 0 { return; }
    if x1 >= FRAME_BUFFER_WIDTH as i16 { return; }
    if y2 < 0 { return; }
    if y1 >= FRAME_BUFFER_HEIGHT as i16 { return; }

    let mut hliney = y1;

    let mut i: u16 = 0;
    let mut j: u16 = (self.num_vertices - 1) as u16;

    x2 = self.vertices[i as usize][0] + x1;
    x1 = self.vertices[j as usize][0] + x1;

    i += 1;
    j -= 1;

    let mut cpt1 = (x1 as u32) << 16;
    let mut cpt2 = (x2 as u32) << 16;

    let mut num_vertices = self.num_vertices;

    loop {
      num_vertices -= 2;

      if num_vertices == 0 {
        break;
      }

      let mut h = self.vertices[j as usize][1] - self.vertices[j as usize + 1][1];
      let mut div = (if h == 0 { 0x4000 } else { 0x4000 / h }) as f32;
      let step1 = ((self.vertices[j as usize][0] - self.vertices[j as usize + 1][0]) as f32 * div * 4.0) as i16;

      h = self.vertices[i as usize][1] - self.vertices[i as usize - 1][1];
      div = (if h == 0 { 0x4000 } else { 0x4000 / h }) as f32;
      let step2 = ((self.vertices[i as usize][0] - self.vertices[i as usize - 1][0]) as f32 * div * 4.0) as i16;

      i += 1;
      j -= 1;

      cpt1 = (cpt1 & 0xFFFF0000) | 0x7FFF;
      cpt2 = (cpt2 & 0xFFFF0000) | 0x8000;

      if h == 0 {
        cpt1 += step1 as u32;
        cpt2 += step2 as u32;
      } else {
        while h != 0 {
          if hliney >= 0 {
            x1 = (cpt1 >> 16) as i16;
            x2 = (cpt2 >> 16) as i16;

            if x1 <= (FRAME_BUFFER_WIDTH - 1) as i16 && x2 >= 0 {
              if x1 < 0 { x1 = 0; }
              if x2 > (FRAME_BUFFER_WIDTH - 1) as i16 { x2 = (FRAME_BUFFER_WIDTH - 1) as i16; }
              self.draw_hor_line(x1, x2, hliney, color, output_pages, backbuffer_page_idx);
            }
          }

          cpt1 = (cpt1 as i32 + step1 as i32) as u32;
          cpt2 = (cpt2 as i32 + step2 as i32) as u32;
          hliney += 1;

          if hliney > (FRAME_BUFFER_HEIGHT - 1) as i16 {
            return;
          }

          h -= 1;
        }
      }
    }
  }

  fn draw_hor_line(&self, x1: i16, x2: i16, y: i16, color: u8, output_pages: &mut [Vec<u8>; 4], backbuffer_page_idx: usize) {
    let xmin = cmp::min(x1, x2);
    let xmax = cmp::max(x1, x2);

    let start_idx = (y as i32 * FRAME_BUFFER_WIDTH as i32 + xmin as i32) as usize;

    for inc in 0..(xmax - (xmin - 1)) {
      let idx = start_idx + inc as usize;

      if color < 0x10 {
        output_pages[backbuffer_page_idx][idx] = color;
      } else if color > 0x10 {
        output_pages[backbuffer_page_idx][idx] = output_pages[0][idx];
      } else {
        output_pages[backbuffer_page_idx][idx] = (output_pages[backbuffer_page_idx][idx] & 0x7) + 0x8;
      }
    }
  }
}
