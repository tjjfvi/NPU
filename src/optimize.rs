use std::collections::{btree_map::Entry, BTreeMap};

use crate::*;

struct Optimizer<'a> {
  fixed: usize,
  redirects: Vec<Option<Wire>>,
  circuit: &'a mut Circuit,
  references: Vec<usize>,
}

impl Circuit {
  pub fn optimize(&mut self) {
    let fixed = 2 + self.inputs;
    let vars = self.wires.len() - fixed;
    let mut optimizer = Optimizer {
      fixed,
      redirects: vec![None; vars],
      references: std::iter::repeat(1)
        .take(fixed)
        .chain(std::iter::repeat(0).take(vars))
        .collect(),
      circuit: self,
    };
    for x in &optimizer.circuit.nands {
      optimizer.references[x.0 .0] += 1;
      optimizer.references[x.1 .0] += 1;
    }
    while optimizer.deduplicate_nands() {}
    optimizer.thin_state();
  }
}

impl Optimizer<'_> {
  pub fn deduplicate_nands(&mut self) -> bool {
    let mut changed = false;
    let mut nands = std::mem::take(&mut self.circuit.nands);
    let mut map = BTreeMap::new();
    nands.retain_mut(|nand| {
      let key = (self.resolve(nand.0), self.resolve(nand.1));
      let mut key = if key.0 .0 < key.1 .0 {
        (key.1, key.0)
      } else {
        key
      };
      let val = self.resolve(nand.2);
      if key.0 == zero() {
        self.merge(one(), val);
        self.references[key.1 .0] -= 1;
        return false;
      }
      if key.0 == one() {
        if key.1 == one() {
          self.merge(zero(), val);
          return false;
        } else {
          key.0 = key.1;
        }
      }
      match match map.entry(key) {
        Entry::Occupied(entry) => Some(*entry.get()),
        Entry::Vacant(entry) => {
          entry.insert(val);
          None
        }
      } {
        Some(existing) => {
          changed = val != existing;
          let val = self.merge(val, existing);
          if val != existing {
            map.insert(key, val);
          }
          return false;
        }
        None => {}
      }
      if key.0 == key.1 {
        map.insert((val, val), key.0);
      }
      *nand = (key.0, key.1, val);
      if self.references[val.0] == 0 {
        changed = true;
        self.references[key.0 .0] -= 1;
        self.references[key.1 .0] -= 1;
        return false;
      }
      true
    });
    self.circuit.nands = nands;
    changed
  }
  pub fn thin_state(&mut self) {
    let mut j = self.fixed;
    for (i, x) in self.redirects.iter_mut().enumerate() {
      if x.is_some() || self.references[i + self.fixed] == 0 {
        *x = None;
        continue;
      }
      *x = Some(Wire(j));
      j += 1;
    }
    let resolve = |w: Wire| {
      w.0
        .checked_sub(self.fixed)
        .map_or(w, |i| self.redirects[i].unwrap())
    };
    for nand in &mut self.circuit.nands {
      nand.0 = resolve(nand.0);
      nand.1 = resolve(nand.1);
      nand.2 = resolve(nand.2);
    }
    self.circuit.wires.splice(j.., []);
    self.references.clear();
    self
      .references
      .extend(std::iter::repeat(0).take(j - self.fixed));
  }
  pub fn resolve(&mut self, wire: Wire) -> Wire {
    let Some(i) = wire.0.checked_sub(self.fixed) else {
      return wire;
    };
    let Some(wire) = self.redirects[i] else {
      return wire;
    };
    let wire = self.resolve(wire);
    self.redirects[i] = Some(wire);
    wire
  }
  pub fn merge(&mut self, a: Wire, b: Wire) -> Wire {
    if a == b {
      return a;
    }
    let (a, b) = if b.0 < a.0 { (b, a) } else { (a, b) };
    self.redirects[b.0 - self.fixed] = Some(a);
    self.references[a.0] += std::mem::take(&mut self.references[b.0]);
    a
  }
}
