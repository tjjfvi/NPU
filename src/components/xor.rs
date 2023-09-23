use crate::*;

impl Circuit {
  pub fn xor<T: Wiring>(&mut self, a: T, b: T, o: T) {
    repeat!(<T> (a, b, o) {
      let (x, ax, bx) = self.wiring();
      self.nand(a, b, x);
      self.nand(a, x, ax);
      self.nand(b, x, bx);
      self.nand(ax, bx, o);
    });
  }
}
