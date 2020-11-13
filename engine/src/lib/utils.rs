pub fn write_u16(mem: &mut [u8], addr: usize, value: u16) {
  mem[addr..addr + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn read_u16(mem: &[u8], addr: u16) -> u16 {
  let mut buffer: [u8; 2] = [0; 2];
  buffer.copy_from_slice(&mem[addr as usize..(addr + 2) as usize]);
  u16::from_be_bytes(buffer)
}

pub fn read_u8(mem: &[u8], addr: u16) -> u8 {
  mem[addr as usize]
}

pub fn read_i16(mem: &[u8], addr: u16) -> i16 {
  let mut buffer: [u8; 2] = [0; 2];
  buffer.copy_from_slice(&mem[addr as usize..(addr + 2) as usize]);
  i16::from_be_bytes(buffer)
}
