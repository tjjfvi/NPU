use crate::*;

impl Circuit {
  pub fn delay_register<T: Wiring>(&mut self, cl: Wire, d: T, o: T) {
    let w = self.wiring();
    self.select(cl, d, w, w);
    self.select(cl, o, w, o);
  }
}
