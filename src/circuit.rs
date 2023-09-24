use std::{
  any::{Any, TypeId},
  cell::RefCell,
  collections::HashMap,
};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wire(pub usize);

#[derive(Debug, Clone)]
pub struct Circuit {
  pub inputs: usize,
  pub wires: Vec<bool>,
  pub nands: Vec<(Wire, Wire, Wire)>,
}

pub fn zero<T: Wiring>() -> T {
  repeat!(<T> () Wire(0))
}
pub fn one<T: Wiring>() -> T {
  repeat!(<T> () Wire(1))
}
pub fn void<T: Wiring>() -> T {
  repeat!(<T> () Wire(usize::MAX))
}

impl Circuit {
  pub fn new(inputs: usize) -> Self {
    let mut state = vec![false; inputs + 2];
    state[1] = true;
    Circuit {
      inputs,
      wires: state,
      nands: vec![],
    }
  }

  pub fn read(&self, w: Wire) -> bool {
    self.wires[w.0]
  }

  pub fn update(&mut self) {
    for n in &self.nands {
      let v = !(self.wires[n.0 .0] && self.wires[n.1 .0]);
      self.wires[n.2 .0] = v;
    }
  }

  pub fn wiring<T: Wiring>(&mut self) -> T {
    repeat!(<T> () {
      let i = self.wires.len();
      self.wires.push(false);
      Wire(i)
    })
  }

  pub fn nand<T: Wiring>(&mut self, a: T, b: T, o: T) {
    repeat!(<T> (a, b, o) {
      if o != void() {
        self.nands.push((a, b, o))
      }
    });
  }

  pub fn num<T: Wiring>(&self, mut n: u64) -> T {
    repeat!(<T> () {
      let w = Wire((n & 1) as usize);
      n >>= 1;
      w
    })
  }
  pub fn read_num<T: Wiring>(&self, t: T) -> u64 {
    let mut n = 0;
    let mut i = 0;
    repeat!(<T> (t) {
      n |= (self.read(t) as u64) << i;
      i += 1;
    });
    n
  }
}

thread_local! {
  static COMPONENT_MEMO: RefCell<HashMap<TypeId, Circuit>> = RefCell::new(HashMap::new());
}

impl Circuit {
  pub fn component<T: Wiring>(&mut self, wiring: T, f: impl FnOnce(&mut Circuit, T) + 'static) {
    let id = f.type_id();
    COMPONENT_MEMO.with(|memo: &RefCell<HashMap<TypeId, Circuit>>| {
      let mut read_lock = memo.borrow();
      let component = if let Some(component) = read_lock.get(&id) {
        component
      } else {
        drop(read_lock);
        let mut inputs = 0;
        let input = repeat!(<T> () {
          inputs += 1;
          Wire(inputs + 1)
        });
        let mut component = Circuit::new(inputs);
        f(&mut component, input);
        component.optimize();

        memo.borrow_mut().insert(id, component);
        read_lock = memo.borrow();
        read_lock.get(&id).unwrap()
      };

      let mut inputs = vec![Wire(0); component.inputs];
      let mut i = 0;
      repeat!(<T> (wiring) {
        inputs[i] = wiring;
        i += 1;
      });
      let start = self.wires.len();
      self
        .wires
        .extend(std::iter::repeat(false).take(component.wires.len() - 2 - component.inputs));
      let translate = |w: Wire| {
        if w.0 < 2 {
          w
        } else {
          inputs
            .get(w.0 - 2)
            .copied()
            .unwrap_or(Wire(start + w.0 - 2 - component.inputs))
        }
      };
      for nand in &component.nands {
        self.nand(translate(nand.0), translate(nand.1), translate(nand.2))
      }
    });
  }
}
