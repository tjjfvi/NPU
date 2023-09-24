use crate::*;

#[component]
pub fn select<T: Wiring>(&mut self, s: Wire, d0: T, d1: T, o: T) {
  let ns = self.wiring();
  self.not(s, ns);
  repeat!(<T> (d0, d1, o) {
    let (x0, x1) = self.wiring();
    self.nand(ns, d0, x0);
    self.nand(s, d1, x1);
    self.nand(x0, x1, o);
  });
}
