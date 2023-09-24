use crate::*;

#[component]
fn _half_add(&mut self, a: Wire, b: Wire, o: Wire, co: Wire) {
  let (x, ax, bx) = self.wiring();
  self.nand(a, b, x);
  self.nand(a, x, ax);
  self.nand(b, x, bx);
  self.nand(ax, bx, o);
  self.not(x, co);
}

#[component]
pub fn add<T: Wiring>(&mut self, a: T, b: T, o: T) {
  let mut c = zero();
  repeat!(<T> (a, b, o) {
    let (o0, c0, c1, c2) = self.wiring();
    self._half_add(a, b, o0, c0);
    self._half_add(o0, c, o, c1);
    self.xor(c0, c1, c2);
    c = c2;
  });
}
