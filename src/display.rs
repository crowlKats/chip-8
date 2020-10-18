pub struct Display {
  pub arr: [[bool; 64]; 32],
}

impl Display {
  pub fn new() -> Display {
    Display {
      arr: [[false; 64]; 32],
    }
  }

  pub fn draw(&mut self, x: usize, y: usize, mem: &[u8]) -> bool {
    let mut unset = false;

    for (i, data) in mem.iter().enumerate() {
      let y = (y + i) % 32;
      for j in 0..8 {
        let x = (x + j) % 64;

        if ((data >> (7 - j as u8)) & 1) == 1 {
          if self.arr[y][x] {
            unset = true;
          }

          self.arr[y][x] ^= true;
        }
      }
    }

    unset
  }

  pub fn clear(&mut self) {
    self.arr = [[false; 64]; 32];
  }
}
