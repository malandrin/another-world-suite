extern crate zip;
extern crate sdl2;
extern crate gl;

use std::os::raw::c_void;
use std::{thread, time};
use gl::types::*;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use awlib::AnotherWorldEngine;
use awlib::opcodes::ActionRequest;
use awlib::defines::{FRAME_BUFFER_WIDTH, FRAME_BUFFER_HEIGHT};

// to run: cargo run --features game
// Althoug the target is the javascript version, I´ve been using this quick and dirty rust version to debug the engine. The engine works fine, but the polygons are rendered broken, I don´t know why.

const WINDOW_WIDTH: i32  = 960;
const WINDOW_HEIGHT: i32 = 600;

fn main() {
  // load zip file
  let filename = format!("./game.zip");
  let path = Path::new(&filename);

  let mut file = match File::open(&path) {
    Err(why) => panic!("can't open {}: {}", path.display(), why.to_string()),
    Ok(file) => file,
  };

  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer).unwrap();

  let reader = std::io::Cursor::new(&buffer);
  let mut zip = zip::ZipArchive::new(reader).unwrap();
  let mut files_dict: HashMap<String, Vec<u8>> = HashMap::new();
  let mut total_size = 0;

  for i in 0..zip.len() {
    let mut file = zip.by_index(i).unwrap();
    let filename = Path::new(file.name()).file_name().unwrap().to_str().unwrap().to_ascii_lowercase();

    if filename == "memlist.bin" || filename.find("bank") == Some(0) {
      let mut file_content: Vec<u8> = Vec::new();
      file.read_to_end(&mut file_content).unwrap();
      let file_len = file_content.len();
      files_dict.insert(filename, file_content);
      total_size += 4 + file_len;
    }
  }

  let mut game_data = vec![0; total_size];

  let mut f = &files_dict["memlist.bin"];
  let mut fs = f.len();

  game_data[0..4].copy_from_slice(&(fs as u32).to_le_bytes());
  game_data[4..4 + fs].clone_from_slice(&f);

  let mut gdidx = 4 + fs;

  for i in 1..14 {
    f = &files_dict[&format!("bank0{:x}", i).to_string()];
    fs = f.len();

    game_data[gdidx..gdidx + 4].copy_from_slice(&(fs as u32).to_le_bytes());
    game_data[gdidx + 4..gdidx + 4 + fs].clone_from_slice(&f);

    gdidx += 4 + fs;
  }

  // initialize sdl
  let sdl = sdl2::init().unwrap();
  let video = sdl.video().unwrap();
  let window =  video.window("Another World", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
    .position_centered()
    .opengl()
    .build()
    .unwrap();

  let _gl_context = window.gl_create_context().unwrap();
  gl::load_with(|s| video.gl_get_proc_address(s) as _);

  let mut texture_id: GLuint = 0;
  let mut fbo_id: GLuint = 0;
  unsafe {
    gl::GenTextures(1, &mut texture_id);
    gl::BindTexture(gl::TEXTURE_2D, texture_id);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

    gl::GenFramebuffers(1, &mut fbo_id);
    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo_id);
    gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
  }

  // start the game
  let mut engine = AnotherWorldEngine::new();

  engine.set_game_data(&game_data);
  engine.init();
  engine.vm_restart(1); // 0xff = protection screen

  let mut event_pump = sdl.event_pump().unwrap();
  let mut paused = false;

  'main: loop {
    for event in event_pump.poll_iter() {
      match event {
          Event::Quit {..} => break 'main,
          Event::KeyDown { keycode: Some(Keycode::P), .. } => paused = !paused,
          Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. } => engine.on_key_down(1),
          Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => engine.on_key_down(2),
          Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => engine.on_key_down(4),
          Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } => engine.on_key_down(8),
          Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. } => engine.on_key_down(16),
          Event::KeyUp { keycode: Some(Keycode::Up), repeat: false, .. } => engine.on_key_up(1),
          Event::KeyUp { keycode: Some(Keycode::Right), repeat: false, .. } => engine.on_key_up(2),
          Event::KeyUp { keycode: Some(Keycode::Down), repeat: false, .. } => engine.on_key_up(4),
          Event::KeyUp { keycode: Some(Keycode::Left), repeat: false, .. } => engine.on_key_up(8),
          Event::KeyUp { keycode: Some(Keycode::Space), repeat: false, .. } => engine.on_key_up(16),
          _ => {},
      }
    }

    if !paused {
      let action_requested = engine.vm_step();

      if (action_requested >> 24) as u8 == ActionRequest::Blit as u8 {
        let frame_buffer_ptr = engine.get_frame_buffer();

        unsafe {
          gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, FRAME_BUFFER_WIDTH as i32, FRAME_BUFFER_HEIGHT as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, frame_buffer_ptr as *const c_void);
          gl::BlitFramebuffer(0, 0, FRAME_BUFFER_WIDTH as i32, FRAME_BUFFER_HEIGHT as i32, 0, WINDOW_HEIGHT, WINDOW_WIDTH, 0, gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }

        window.gl_swap_window();

        thread::sleep(time::Duration::from_millis(16));
      }
    }
  }

  unsafe {
    gl::DeleteFramebuffers(1, &mut fbo_id);
  }
}