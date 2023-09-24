use crate::*;

#[component]
pub fn or<T: Wiring>(&mut self, a: T, b: T, o: T) {
  repeat!(<T> (a, b, o) {
    let (na, nb) = self.wiring();
    self.not(a, na);
    self.not(b, nb);
    self.nand(na, nb, o);
  });
}
