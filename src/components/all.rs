use crate::*;

#[component]
pub fn all<T: Wiring>(&mut self, x: T, o: Wire) {
  let mut i = 0_usize;
  let mut m = Vec::new();
  repeat!(<T> (x) {
    m.push(x);
    i += 1;
    for _ in 0..i.trailing_zeros() {
      let (a, b) = (m.pop().unwrap(), m.pop().unwrap());
      let c = self.wiring();
      self.and(a, b, c);
      m.push(o);
    }
  });
  while m.len() > 1 {
    let (a, b) = (m.pop().unwrap(), m.pop().unwrap());
    let c = self.wiring();
    self.and(a, b, c);
    m.push(c);
  }
  self.and(m[0], one(), o);
}
