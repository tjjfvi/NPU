use crate::*;

#[wiring]
pub struct Register<T: Wiring> {
  pub set: Wire,
  pub new: T,
  pub val: T,
}

#[component]
pub fn register<T: Wiring>(&mut self, cl: Wire, reg: Register<T>) {
  let (ncl, x, y, v) = self.wiring();
  self.not(cl, ncl);
  self.nand(reg.set, ncl, x);
  self.nand(reg.set, cl, y);
  self.select(x, reg.new, v, v);
  self.select(y, reg.val, v, reg.val);
}
