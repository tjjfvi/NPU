#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod bundle;
mod circuit;
mod components;

use bundle::*;
use circuit::*;

use std::time::Instant;

fn main() {
  let mut circuit = Circuit::new();
  let nums = flat(&[[1, 5, 3, 2, 0, 9, 8, 4, 7, 6]; 800]).map(|x| circuit.num::<[Wire; 32]>(x));
  let start = Instant::now();
  let out = circuit.sort(nums);
  println!("{:?} nands", circuit.nands.len());
  println!("initialize {:?}", Instant::now() - start);
  let start = Instant::now();
  circuit.update();
  println!("update {:?}", Instant::now() - start);
  // dbg!(out.map(|x| circuit.read_num(x)));
}

fn flat<T, const X: usize, const Y: usize>(a: &[[T; X]; Y]) -> &[T; X * Y] {
  unsafe { std::mem::transmute(a) }
}
