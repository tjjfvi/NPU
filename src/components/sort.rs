use crate::*;

#[component]
fn _sort_2<T: Wiring>(&mut self, a: T, b: T, x: T, y: T) {
  let s = self.wiring();
  self.lt(b, a, s);
  self.swap(s, a, b, x, y);
}

impl Circuit {
  pub fn sort<T: Wiring, const N: usize>(&mut self, mut arr: [T; N]) -> [T; N] {
    for (pi, p) in (0..).map(|i| (i, 1 << i)).take_while(|(_, p)| p < &N) {
      for k in (0..=pi).rev().map(|i| 1 << i) {
        for j in ((k % p)..N - k).step_by(2 * k) {
          for i in 0..usize::min(k, N - j - k) {
            let a = i + j;
            let b = a + k;
            if a >> (pi + 1) == b >> (pi + 1) {
              let (x, y) = self.wiring();
              self._sort_2(arr[a], arr[b], x, y);
              (arr[a], arr[b]) = (x, y);
            }
          }
        }
      }
    }
    arr
  }
}
