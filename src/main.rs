#![feature(generic_const_exprs)]

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wire(usize);

pub struct Circuit {
  state: Vec<bool>,
  nands: Vec<(Wire, Wire, Wire)>,
}

macro_rules! with_ignore {
  ($ignore:tt $($out:tt)*) => { $($out)* };
}

macro_rules! repeat {
  (<$T:ident> ($($x:ident),* $(,)?) $r:expr) => {
    $T::Bundle::map(
      $T::Bundle::transpose::<Wire, ($(with_ignore!($x Item),)*)>(($($x,)*)),
      |($($x,)*)| $r
    )
  };
}

impl Circuit {
  pub fn new() -> Self {
    Circuit {
      state: vec![false, true],
      nands: vec![],
    }
  }
  pub fn read(&self, w: Wire) -> bool {
    self.state[w.0]
  }
  pub fn update(&mut self) {
    for n in &self.nands {
      let v = !(self.state[n.0 .0] && self.state[n.1 .0]);
      self.state[n.2 .0] = v;
    }
  }
  pub fn wire(&mut self) -> Wire {
    let i = self.state.len();
    self.state.push(false);
    Wire(i)
  }
  pub fn wiring<T: Wiring>(&mut self) -> T {
    repeat!(<T> () self.wire())
  }
  pub fn nand(&mut self, a: Wire, b: Wire, o: Wire) {
    if o != self.void() {
      self.nands.push((a, b, o))
    }
  }
  pub fn not<T: Wiring>(&mut self, i: T, o: T) {
    repeat!(<T> (i, o) self.nand(i, i, o));
  }
  pub fn and(&mut self, a: Wire, b: Wire, o: Wire) {
    let x = self.wiring();
    self.nand(a, b, x);
    self.not(x, o)
  }
  pub fn or(&mut self, a: Wire, b: Wire, o: Wire) {
    let (na, nb) = self.wiring();
    self.not(a, na);
    self.not(b, nb);
    self.nand(na, nb, o);
  }
  pub fn xor(&mut self, a: Wire, b: Wire, o: Wire) {
    let (x, ax, bx) = self.wiring();
    self.nand(a, b, x);
    self.nand(a, x, ax);
    self.nand(b, x, bx);
    self.nand(ax, bx, o);
  }
  pub fn select1(&mut self, s: Wire, d0: Wire, d1: Wire, o: Wire) {
    let (ns, x0, x1) = self.wiring();
    self.not(s, ns);
    self.nand(ns, d0, x0);
    self.nand(s, d1, x1);
    self.nand(x0, x1, o);
  }
  pub fn swap<T: Wiring>(&mut self, s: Wire, a: T, b: T, x: T, y: T) {
    repeat!(<T> (a, b, x, y) {
      self.select1(s, a, b, x);
      self.select1(s, b, a, y);
    });
  }
  pub fn register<T: Wiring>(&mut self, cl: Wire, st: Wire, d: T, o: T) {
    let (ncl, x, s) = self.wiring();
    self.not(cl, ncl);
    self.nand(st, ncl, x);
    repeat!(<T> (s, d, o) {
      self.select1(x, d, o, s);
      self.select1(cl, o, s, o);
    });
  }
  pub fn half_add(&mut self, a: Wire, b: Wire, o: Wire, co: Wire) {
    let (x, ax, bx) = self.wiring();
    self.nand(a, b, x);
    self.nand(a, x, ax);
    self.nand(b, x, bx);
    self.nand(ax, bx, o);
    self.not(x, co);
  }
  pub fn add_wire(&mut self, a: Wire, b: Wire, c: Wire, o: Wire, co: Wire) {
    let (o0, c0, c1) = self.wiring();
    self.half_add(a, b, o0, c0);
    self.half_add(o0, c, o, c1);
    self.xor(c0, c1, co);
  }
  pub fn add<T: Wiring>(&mut self, a: T, b: T, o: T) {
    let mut c = self.zero();
    repeat!(<T> (a, b, o) {
      let co = self.wire();
      self.add_wire(a, b, c, o, co);
      c = co;
    });
  }
  pub fn zero<T: Wiring>(&self) -> T {
    repeat!(<T> () Wire(0))
  }
  pub fn one<T: Wiring>(&self) -> T {
    repeat!(<T> () Wire(1))
  }
  pub fn void<T: Wiring>(&self) -> T {
    repeat!(<T> () Wire(usize::MAX))
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
  pub fn lt<T: Wiring + Copy>(&mut self, a: T, b: T, o: Wire) {
    let (na, s) = self.wiring();
    let mut nab = self.void();
    *last(&mut nab) = s;
    self.not(a, na);
    self.add(na, b, nab);
    self.not(s, o);
  }
  pub fn sort2<T: Wiring + Copy>(&mut self, a: T, b: T, x: T, y: T) {
    let s = self.wiring();
    self.lt(b, a, s);
    self.swap(s, a, b, x, y);
  }
  pub fn sort<T: Wiring + Copy, const N: usize>(&mut self, mut arr: [T; N]) -> [T; N] {
    for (pi, p) in (0..).map(|i| (i, 1 << i)).take_while(|(_, p)| p < &N) {
      for k in (0..=pi).rev().map(|i| 1 << i) {
        for j in ((k % p)..N - k).step_by(2 * k) {
          for i in 0..usize::min(k, N - j - k) {
            let a = i + j;
            let b = a + k;
            if a >> (pi + 1) == b >> (pi + 1) {
              let (x, y) = self.wiring();
              self.sort2(arr[a], arr[b], x, y);
              (arr[a], arr[b]) = (x, y);
            }
          }
        }
      }
    }
    arr
  }
}

pub struct Item;

pub trait Bundle {
  type Of<T>;
  fn as_mut<T>(t: &mut Self::Of<T>) -> Self::Of<&mut T>;
  fn map<T, U>(t: Self::Of<T>, f: impl FnMut(T) -> U) -> Self::Of<U>;
  fn transpose<T, B: Bundle>(b: B::Of<Self::Of<T>>) -> Self::Of<B::Of<T>>;
}

pub trait Wiring {
  type Bundle: Bundle<Of<Wire> = Self>;
}

impl Wiring for Wire {
  type Bundle = Item;
}

impl Bundle for Item {
  type Of<T> = T;
  #[inline(always)]
  fn as_mut<T>(t: &mut Self::Of<T>) -> Self::Of<&mut T> {
    t
  }
  #[inline(always)]
  fn map<T, U>(t: Self::Of<T>, mut f: impl FnMut(T) -> U) -> Self::Of<U> {
    f(t)
  }
  #[inline(always)]
  fn transpose<T, B: Bundle>(b: B::Of<Self::Of<T>>) -> Self::Of<B::Of<T>> {
    b
  }
}

macro_rules! impl_tuple {
  ($($i:tt $T:ident)*) => {
    #[allow(unused)]
    impl<$($T: Bundle,)*> Bundle for ($($T,)*) {
      type Of<T> = ($($T::Of<T>,)*);
      #[inline(always)]
      fn as_mut<T>(t: &mut Self::Of<T>) -> Self::Of<&mut T> {
        ($($T::as_mut(&mut t.$i),)*)
      }
      #[inline(always)]
      fn map<T, U>(t: Self::Of<T>, mut f: impl FnMut(T) -> U) -> Self::Of<U> {
        ($($T::map(t.$i, &mut f),)*)
      }
      #[inline(always)]
      fn transpose<T, X: Bundle>(x: X::Of<Self::Of<T>>) -> Self::Of<X::Of<T>> {
        let mut x = X::map(x, |a| ($(Some(a.$i),)*));
        ($($T::transpose::<T, X>(X::map(X::as_mut(&mut x), |a| a.$i.take().unwrap())),)*)
      }
    }
    impl<$($T: Wiring,)*> Wiring for ($($T,)*) {
      type Bundle = ($($T::Bundle,)*);
    }
  };
}

impl_tuple!();
impl_tuple!(0 A);
impl_tuple!(0 A 1 B);
impl_tuple!(0 A 1 B 2 C);
impl_tuple!(0 A 1 B 2 C 3 D);
impl_tuple!(0 A 1 B 2 C 3 D 4 E);
impl_tuple!(0 A 1 B 2 C 3 D 4 E 5 F);

impl<X: Bundle, const N: usize> Bundle for [X; N] {
  type Of<T> = [X::Of<T>; N];
  #[inline(always)]
  fn map<T, U>(t: Self::Of<T>, mut f: impl FnMut(T) -> U) -> Self::Of<U> {
    t.map(|x| X::map(x, &mut f))
  }
  #[inline(always)]
  fn as_mut<T>(t: &mut Self::Of<T>) -> Self::Of<&mut T> {
    let mut i = t.iter_mut();
    [(); N].map(|_| X::as_mut(i.next().unwrap()))
  }
  #[inline(always)]
  fn transpose<T, B: Bundle>(x: B::Of<Self::Of<T>>) -> Self::Of<B::Of<T>> {
    let mut x = B::map(x, |a| a.map(Some));
    let mut i = 0;
    [(); N].map(|_| {
      let x = X::transpose::<T, B>(B::map(B::as_mut(&mut x), |a| a[i].take().unwrap()));
      i += 1;
      x
    })
  }
}

impl<X: Wiring, const N: usize> Wiring for [X; N] {
  type Bundle = [X::Bundle; N];
}

fn last<T: Wiring>(t: &mut T) -> &mut Wire {
  let mut last = None;
  T::Bundle::map(T::Bundle::as_mut(t), |x: &mut Wire| last = Some(x));
  last.unwrap()
}

fn main() {
  let mut circuit = Circuit::new();
  let nums = flat(&[[1, 5, 3, 2, 0, 9, 8, 4, 7, 6]; 30]).map(|x| circuit.num::<[Wire; 32]>(x));
  let start = Instant::now();
  let out = circuit.sort(nums);
  println!("{:?} nands", circuit.nands.len());
  println!("initialize {:?}", Instant::now() - start);
  let start = Instant::now();
  circuit.update();
  println!("update {:?}", Instant::now() - start);
  dbg!(out.map(|x| circuit.read_num(x)));
}

fn flat<T, const X: usize, const Y: usize>(a: &[[T; X]; Y]) -> &[T; X * Y] {
  unsafe { std::mem::transmute(a) }
}
