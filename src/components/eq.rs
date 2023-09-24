use crate::*;

#[component]
pub fn eq<T: Wiring>(&mut self, a: T, b: T, o: Wire) {
  let (x, y) = self.wiring();
  self.xor(a, b, x);
  self.not(x, y);
  self.all(y, o);
}
