use crate::*;

impl Circuit {
  pub fn lt<T: Wiring>(&mut self, a: T, b: T, o: Wire) {
    let (na, s) = self.wiring();
    let mut nab = self.void();
    *last(&mut nab) = s;
    self.not(a, na);
    self.add(na, b, nab);
    self.not(s, o);
  }
}
