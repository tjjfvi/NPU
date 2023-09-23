use crate::*;

impl Circuit {
  pub fn and<T: Wiring>(&mut self, a: T, b: T, o: T) {
    repeat!(<T> (a, b, o) {
      let x = self.wiring();
      self.nand(a, b, x);
      self.not(x, o)
    });
  }
}
