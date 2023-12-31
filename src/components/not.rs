use crate::*;

#[component]
pub fn not<T: Wiring>(&mut self, i: T, o: T) {
  repeat!(<T> (i, o) self.nand(i, i, o));
}
