use {dbg::msg, std::iter::repeat_with};

#[derive(Debug, Clone, PartialEq)]
pub enum Veb {
    Base(Base),
    Rec(Rec),
}
impl Veb {
    pub fn len(&self) -> usize {
        1 << self.lg()
    }
    pub fn new(lg: u32) -> Self {
        if lg == 1 {
            Veb::Base(Base::new())
        } else {
            Veb::Rec(Rec::new(lg))
        }
    }
    pub fn contains(&self, x: usize) -> bool {
        match self {
            Veb::Base(base) => base.contains(x),
            Veb::Rec(rec) => rec.contains(x),
        }
    }
    pub fn min(&self) -> Option<usize> {
        match self {
            Veb::Base(base) => base.min(),
            Veb::Rec(rec) => rec.min(),
        }
    }
    pub fn succ(&self, x: usize) -> Option<usize> {
        match self {
            Veb::Base(base) => base.succ(x),
            Veb::Rec(rec) => rec.succ(x),
        }
    }
    pub fn insert(&mut self, x: usize) {
        match self {
            Veb::Base(base) => base.insert(x),
            Veb::Rec(rec) => rec.insert(x),
        }
    }
    pub fn collect_bitvec(&self) -> Vec<bool> {
        let mut vec = vec![false; 1 << self.lg()];
        self.copy_bitvec(&mut vec);
        vec
    }
    fn copy_bitvec(&self, vec: &mut [bool]) {
        match self {
            Veb::Base(base) => base.copy_bitvec(vec),
            Veb::Rec(rec) => rec.copy_bitvec(vec),
        }
    }
    fn lg(&self) -> u32 {
        match self {
            Veb::Base(_) => 1,
            Veb::Rec(rec) => rec.lg,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Base([bool; 2]);
impl Base {
    fn new() -> Self {
        Self([false; 2])
    }
    fn contains(&self, x: usize) -> bool {
        assert!(x < 2);
        self.0[x]
    }
    fn min(&self) -> Option<usize> {
        self.0.iter().position(|&b| b)
    }
    fn succ(&self, x: usize) -> Option<usize> {
        if x == 0 && self.0[1] {
            Some(1)
        } else {
            None
        }
    }
    fn insert(&mut self, x: usize) {
        self.0[x] = true
    }
    fn copy_bitvec(&self, vec: &mut [bool]) {
        vec.copy_from_slice(&self.0);
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Rec {
    lg: u32,
    lower: u32,
    summary: Box<Veb>,
    cluster: Vec<Veb>,
}
impl Rec {
    fn new(lg: u32) -> Self {
        assert!(1 < lg);
        let lower = lg / 2;
        let upper = lg - lower;
        Self {
            lg,
            lower,
            summary: Box::new(Veb::new(upper)),
            cluster: repeat_with(|| Veb::new(lower)).take(1 << upper).collect(),
        }
    }
    fn contains(&self, x: usize) -> bool {
        assert!(x < 1 << self.lg);
        let (high, low) = self.decompose(x);
        self.cluster[high].contains(low)
    }
    fn min(&self) -> Option<usize> {
        let high = self.summary.min()?;
        let low = self.cluster[high].min()?;
        Some(self.index(high, low))
    }
    fn succ(&self, x: usize) -> Option<usize> {
        let (high, low) = self.decompose(x);
        self.cluster[high]
            .succ(low)
            .map(|low| self.index(high, low))
            .or_else(|| {
                let high = self.summary.succ(high)?;
                let low = self.cluster[high].min()?;
                Some(self.index(high, low))
            })
    }
    fn insert(&mut self, x: usize) {
        let (high, low) = self.decompose(x);
        self.summary.insert(high);
        self.cluster[high].insert(low);
    }
    fn copy_bitvec(&self, vec: &mut [bool]) {
        self.cluster.iter().enumerate().for_each(|(i, child)| {
            let range = child.len() * i..child.len() * (i + 1);
            child.copy_bitvec(&mut vec[range])
        });
    }
    fn index(&self, high: usize, low: usize) -> usize {
        (high << self.lower) + low
    }
    fn decompose(&self, x: usize) -> (usize, usize) {
        (
            x >> self.lower,
            x & (std::usize::MAX >> std::mem::size_of::<usize>() as u32 * 8 - self.lower),
        )
    }
}

#[cfg(test)]
mod test {
    use {
        super::{Rec, Veb},
        dbg::BooleanSlice,
        rand::prelude::*,
        std::collections::BTreeSet,
        yansi::Paint,
    };

    #[test]
    fn test_decompose() {
        let rec = Rec::new(4);
        assert_eq!(rec.decompose(10), (2, 2));
    }

    #[test]
    fn test() {
        let mut test = Test::new(4);
        test.insert(2);
        test.insert(3);
        test.insert(4);
        test.insert(5);
        test.insert(7);
        test.insert(14);
        test.insert(15);
    }

    #[test]
    fn test_rand() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let lg = rng.gen_range(1, 5);
            let mut test = Test::new(lg);
            let len = 1 << lg;
            for _ in 0..100 {
                match rng.gen_range(0, 4) {
                    0 => test.contains(rng.gen_range(0, len)),
                    1 => test.min(),
                    2 => test.succ(rng.gen_range(0, len)),
                    3 => test.insert(rng.gen_range(0, len)),
                    _ => unreachable!(),
                }
            }
        }
    }
    struct Test {
        veb: Veb,
        set: BTreeSet<usize>,
    }
    impl Test {
        fn new(lg: u32) -> Self {
            let res = Self {
                veb: Veb::new(lg),
                set: BTreeSet::new(),
            };
            res.postproces();
            res
        }
        fn contains(&self, x: usize) {
            println!("{}: {:?}", Paint::yellow("Red").bold(), x);
            assert_eq!(self.veb.contains(x), self.set.contains(&x));
            self.postproces();
        }
        fn min(&self) {
            println!("{}", Paint::magenta("Min").bold());
            assert_eq!(self.veb.min(), self.set.iter().next().copied());
            self.postproces();
        }
        fn succ(&self, x: usize) {
            println!("{}: {:?}", Paint::yellow("Succ").bold(), x);
            assert_eq!(self.veb.succ(x), self.set.range(x + 1..).next().copied());
            self.postproces();
        }
        fn insert(&mut self, x: usize) {
            println!("{}: {:?}", Paint::green("Insert").bold(), x);
            self.veb.insert(x);
            self.set.insert(x);
            self.postproces();
        }
        fn postproces(&self) {
            println!("set = {:?}", &self.set);
            println!("veb = {:?}", BooleanSlice(&self.veb.collect_bitvec()));
        }
    }
}
