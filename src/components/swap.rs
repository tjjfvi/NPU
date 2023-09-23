use crate::*;

impl Circuit {
  pub fn swap<T: Wiring>(&mut self, s: Wire, a: T, b: T, x: T, y: T) {
    self.select(s, a, b, x);
    self.select(s, b, a, y);
  }
}
