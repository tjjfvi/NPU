
type Wire = { power: boolean, not?: Wire }

const nands: { a: Wire, b: Wire, out: Wire }[] = []

const zero: Wire = { power: false }
const one: Wire = { power: true }

zero.not = one
one.not = zero

const nand = (a: Wire, b: Wire) => {
  const out = { power: !(a.power && b.power) }
  nands.push({ a, b, out })
  return out
}

const not = (x: Wire)  => {
  if(!x.not){
    x.not = nand(x, x)
    x.not.not = x
  }
  return x.not
}

const and = (a: Wire, b: Wire) => not(nand(a, b))

const or = (a: Wire, b: Wire) => nand(not(a), not(b))

const xor = (a: Wire, b: Wire) => {
  const x = nand(a, b)
  return nand(nand(a, x), nand(b, x))
}

const select1 = (s: Wire, d0: Wire, d1: Wire): Wire =>
  nand(nand(s, d1), nand(not(s), d0))

const swap1 = (s: Wire, a: Wire, b: Wire): [Wire, Wire] => 
  [nand(nand(s, b), nand(not(s), a)), nand(nand(s, a), nand(not(s), b))]

const fix = (f: (x: Wire) => Wire) => {
  const r = f({ get power() { return r.power }})
  return r
}

const dff = (cl: Wire, st: Wire, d: Wire): Wire => fix(out =>
  select1(cl, out, select1(nand(st, not(cl)), d, out))
)

const halfadd = (a: Wire, b: Wire) : [Wire, Wire]=> {
  const x = nand(a, b) 
  return [
    not(x),
    nand(nand(a, x), nand(b, x))
  ]
}

const add1 = (a: Wire, b: Wire, c: Wire): [Wire, Wire] => {
  const [x, y] = halfadd(a, b)
  const [z, w] = halfadd(y, c)
  return [xor(x, z), w]
}

const add = (a: Wire[], b: Wire[], c: Wire = zero) => {
  const o: Wire[] = []
  for(let i = 0; i < a.length || i < b.length; i++) {
    const [c2, x] = add1(a[i] ?? zero, b[i] ?? zero, c)
    c = c2
    o.push(x)
  }
  return o
}

const num = (x: number, l = 0): Wire[] => {
  const o: Wire[] = Array(l).fill(zero)
  for(let i = 0; x; i++) {
    o[i] = x & 1 ? one : zero
    x >>= 1
  }
  return o
}

const readNum = (x: Wire[]): number => {
  let n = 0
  for(let i = 0; i < x.length; i++) {
    n |= +x[i].power << i
  }
  return n
}

const lt = (a: Wire[], b: Wire[]) => not(add(a.map(not), b).at(-1)!)

type Addr = Wire[]
type Message = Wire[]
type Dual<T> = (x: T) => void

type Link = {
  out: Message, 
  in: Dual<Message>,
}

type Port = {
  addr: Addr,
  wire: Link,
  lane: Link
}

const swapLink = (s: Wire, a: Link, b: Link): [Link, Link] => {
  const [xO, yO] = transpose(transpose([a.out, b.out]).map(([a, b]) => swap1(s, a, b)))
  let xI: Message | undefined;
  let yI: Message | undefined;
  return [
    {out: xO, in: (i) => { xI = i; if(yI) connect() }},
    {out: yO, in: (i) => { yI = i; if(xI) connect() }},
  ]
  function connect(){
    const [aI, bI] = transpose(transpose([xI!, yI!]).map(([x, y]) => swap1(s, x, y)))
    a.in(aI)
    b.in(bI)
  }
}

let n = 0
const sort2 = (a: Port, b: Port): [Port, Port] => {
  const s = lt(b.addr, a.addr)
  n++
  const [xA, yA] = transpose(transpose([a.addr, b.addr]).map(([a, b]) => swap1(s, a, b)))
  const [xW, yW] = swapLink(s, a.wire, b.wire)
  const [xL, yL] = swapLink(s, a.lane, b.lane)
  return [
    { addr: xA, wire: xW, lane: xL, },
    { addr: yA, wire: yW, lane: yL, }
  ]
}

const sort = (a: Port[]): Port[] => {
  a = a.slice()
  const n = a.length
  for(let p = 1; p < n; p *= 2)
    for(let k = p; k; k >>= 1)
      for(let j = k % p; j <= n - 1 - k; j += 2 * k)
        for(let i = 0; i <= Math.min(k-1, n-j-k-1); i++)
          if(Math.floor((i + j) / (p * 2) ) === Math.floor((i + j + k) / (p * 2) ))
            [a[i + j], a[i + j + k]] = sort2(a[i + j], a[i + j + k])
  return a
}

const router = (p: Port[]) => {
  p = sort(p)
  for(let i = 0; i < p.length; i += 2) {
    link(p[i].wire, p[i+1].wire)
  }
  for(let i = 0; i < p.length / 2; i++) {
    link(p[i].lane, p[p.length - 1 - i].lane)
  }

  function link(a: Link, b: Link) {
    a.in(b.out)
    b.in(a.out)
  }
}

const link = (): Link => ({ in: () => {}, out: num(0, 0) })

const transpose = <T>(x: T[][]): T[][] => x[0]?.map((_, i) => x.map(a => a[i])) ?? []

const port = (n: number) => ({ addr: num(n, 32), lane: link(), wire: link() })

add(num(0, 5), num(0, 5), zero)

console.log(nands.length)

console.time("generation")
let x = sort(Array(30).fill([1,5,3,2,0,9,8,4,7,6]).flat().map(port))
console.timeEnd("generation")

console.time("update")
update()
console.timeEnd("update")

console.log(nands.length)

console.log(n)

function update() {
  for(const g of nands) {
    g.out.power = !(g.a.power && g.b.power)
  }
}

