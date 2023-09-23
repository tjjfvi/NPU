use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wire(usize);

#[derive(Debug, Clone)]
pub struct Circuit {
  pub state: Vec<bool>,
  pub nands: Vec<(Wire, Wire, Wire)>,
}

impl Circuit {
  pub fn new() -> Self {
    Circuit {
      state: vec![false, true],
      nands: vec![],
    }
  }

  pub fn read(&self, w: Wire) -> bool {
    self.state[w.0]
  }

  pub fn update(&mut self) {
    for n in &self.nands {
      let v = !(self.state[n.0 .0] && self.state[n.1 .0]);
      self.state[n.2 .0] = v;
    }
  }

  pub fn wiring<T: Wiring>(&mut self) -> T {
    repeat!(<T> () {
      let i = self.state.len();
      self.state.push(false);
      Wire(i)
    })
  }
  pub fn zero<T: Wiring>(&self) -> T {
    repeat!(<T> () Wire(0))
  }
  pub fn void<T: Wiring>(&self) -> T {
    repeat!(<T> () Wire(usize::MAX))
  }

  pub fn nand<T: Wiring>(&mut self, a: T, b: T, o: T) {
    repeat!(<T> (a, b, o) {
      if o != self.void() {
        self.nands.push((a, b, o))
      }
    });
  }

  pub fn num<T: Wiring>(&self, mut n: u64) -> T {
    repeat!(<T> () {
      let w = Wire((n & 1) as usize);
      n >>= 1;
      w
    })
  }
  pub fn read_num<T: Wiring>(&self, t: T) -> u64 {
    let mut n = 0;
    let mut i = 0;
    repeat!(<T> (t) {
      n |= (self.read(t) as u64) << i;
      i += 1;
    });
    n
  }
}
