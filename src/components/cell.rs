use crate::*;

type Tag = (Wire, Wire);
type Id<const N: usize> = [Wire; N];
type Addr<const N: usize> = (Tag, Id<N>, Wire);

type State<const N: usize> = (Wire, Id<N>, [Addr<N>; 3]);

type Cell<const N: usize> = ([Addr<N>; 3], [Addr<N>; 3]);

impl Circuit {
  fn cell<const N: usize>(&mut self, cl: Wire, p: (Wire, Wire), (msg_out, _msg_in): Cell<N>) {
    let (set_state, state) = self.wiring();
    self.delay_register::<State<N>>(cl, set_state, state);
    let msg_out_p01 = {
      let msg_0 = self.wiring();
      self.select(p.0, state.2[0], state.2[2], msg_0);
      [msg_0, state.2[1], state.2[2]]
    };
    let msg_out_p23 = {
      let msg_0 = self.wiring();
      let msg_0_p2 = ((one(), state.0), state.1, zero());
      let msg_0_p3 = self.wiring();
      self.select(p.0, msg_0_p2, msg_0_p3, msg_0);
      let msg_1 = ((zero(), state.0), state.2[2].1, state.2[2].2);
      let msg_2 = zero();
      [msg_0, msg_1, msg_2]
    };
    self.select(p.1, msg_out_p01, msg_out_p23, msg_out);
  }
}
