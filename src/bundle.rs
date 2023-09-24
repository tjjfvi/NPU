use crate::*;

#[macro_export]
macro_rules! with_ignore {
  ($ignore:tt $($out:tt)*) => { $($out)* };
}

#[macro_export]
macro_rules! repeat {
  (<$A:ident> ($($x:ident),* $(,)?) $r:expr) => {
    $A::map(
      $A::transpose::<Wire, ($(with_ignore!($x Wire),)*)>(($($x,)*)),
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

pub trait Wiring: Copy + Bundle<Of<Wire> = Self> + 'static {}
impl<T: Copy + Bundle<Of<Wire> = Self> + 'static> Wiring for T {}

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

#[macro_export]
macro_rules! impl_bundle {
  (($($i:tt $A:ident),*)) => {
    impl_bundle!(($($A: Bundle),*), ($($A,)*), (), ($($A::Of<X>,)*), ($($i $A),*));
  };
  (($($g:tt)*), $T:ty, ($($w:tt)*), $Of:ty, $($name:ident)? ($($i:tt $A:ty),*)) => {
    #[allow(unused)]
    impl<$($g)*> Bundle for $T $($w)* {
      type Of<X> = $Of;
      #[inline(always)]
      fn as_mut<X>(t: &mut Self::Of<X>) -> Self::Of<&mut X> {
        $($name)? ($(<$A as Bundle>::as_mut(&mut t.$i),)*)
      }
      #[inline(always)]
      fn map<X, Y>(t: Self::Of<X>, mut f: impl FnMut(X) -> Y) -> Self::Of<Y> {
        $($name)? ($(<$A as Bundle>::map(t.$i, &mut f),)*)
      }
      #[inline(always)]
      fn transpose<X, Y: Bundle>(x: Y::Of<Self::Of<X>>) -> Self::Of<Y::Of<X>> {
        let mut x = Y::map(x, |a| ($(Some(a.$i),)*));
        $($name)? ($(<$A as Bundle>::transpose::<X, Y>(Y::map(Y::as_mut(&mut x), |a| a.$i.take().unwrap())),)*)
      }
    }
  };
  (($($g:tt)*), $T:ty, ($($w:tt)*), $Of:ty, $name:ident { $($i:tt $k:ident: $A:ty),* }) => {
    #[allow(unused)]
    impl<$($g)*> Bundle for $T $($w)* {
      type Of<X> = $Of;
      #[inline(always)]
      fn as_mut<X>(t: &mut Self::Of<X>) -> Self::Of<&mut X> {
        $name { $($k: <$A as Bundle>::as_mut(&mut t.$k),)* }
      }
      #[inline(always)]
      fn map<X, Y>(t: Self::Of<X>, mut f: impl FnMut(X) -> Y) -> Self::Of<Y> {
        $name { $($k: <$A as Bundle>::map(t.$k, &mut f),)* }
      }
      #[inline(always)]
      fn transpose<X, Y: Bundle>(x: Y::Of<Self::Of<X>>) -> Self::Of<Y::Of<X>> {
        let mut x = Y::map(x, |a| ($(Some(a.$k),)*));
        $name { $($k: <$A as Bundle>::transpose::<X, Y>(Y::map(Y::as_mut(&mut x), |a| a.$i.take().unwrap())),)* }
      }
    }
  };
}

impl_bundle!(());
impl_bundle!((0 A));
impl_bundle!((0 A, 1 B));
impl_bundle!((0 A, 1 B, 2 C));
impl_bundle!((0 A, 1 B, 2 C, 3 D));
impl_bundle!((0 A, 1 B, 2 C, 3 D, 4 E));
impl_bundle!((0 A, 1 B, 2 C, 3 D, 4 E, 5 F));

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
