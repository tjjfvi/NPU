use crate::*;

impl Circuit {
  pub fn register<T: Wiring>(&mut self, cl: Wire, st: Wire, d: T, o: T) {
    let (ncl, x, s) = self.wiring();
    self.not(cl, ncl);
    self.nand(st, ncl, x);
    self.select(x, d, o, s);
    self.select(cl, o, s, o);
  }
}
