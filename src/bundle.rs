use crate::*;

#[macro_export]
macro_rules! with_ignore {
  ($ignore:tt $($out:tt)*) => { $($out)* };
}

#[macro_export]
macro_rules! repeat {
  (<$T:ident> ($($x:ident),* $(,)?) $r:expr) => {
    $T::map(
      $T::transpose::<Wire, ($(with_ignore!($x Wire),)*)>(($($x,)*)),
      |($($x,)*)| $r
    )
  };
}

pub trait Bundle {
  type Of<T>;
  fn as_mut<T>(t: &mut Self::Of<T>) -> Self::Of<&mut T>;
  fn map<T, U>(t: Self::Of<T>, f: impl FnMut(T) -> U) -> Self::Of<U>;
  fn transpose<T, B: Bundle>(b: B::Of<Self::Of<T>>) -> Self::Of<B::Of<T>>;
}

pub trait Wiring: Copy + Bundle<Of<Wire> = Self> {}
impl<T: Copy + Bundle<Of<Wire> = Self>> Wiring for T {}

impl Bundle for Wire {
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

pub fn last<T: Wiring>(t: &mut T) -> &mut Wire {
  let mut last = None;
  T::map(T::as_mut(t), |x: &mut Wire| last = Some(x));
  last.unwrap()
}
