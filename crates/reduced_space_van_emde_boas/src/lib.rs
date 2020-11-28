use std::{
    collections::HashMap,
    mem::{size_of, swap},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Veb {
    Base(Base),
    Rec(Rec),
}
impl Veb {
    pub fn new(lg: u32) -> Self {
        if lg <= 6 {
            Veb::Base(Base::new(lg))
        } else {
            Veb::Rec(Rec::new(lg))
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Veb::Base(base) => base.is_empty(),
            Veb::Rec(rec) => rec.is_empty(),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Veb::Base(_) => 2,
            Veb::Rec(rec) => 1 << rec.lg,
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
    pub fn max(&self) -> Option<usize> {
        match self {
            Veb::Base(base) => base.max(),
            Veb::Rec(rec) => rec.max(),
        }
    }
    pub fn prev(&self, x: usize) -> Option<usize> {
        match self {
            Veb::Base(base) => base.prev(x),
            Veb::Rec(rec) => rec.prev(x),
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
    pub fn delete(&mut self, x: usize) -> bool {
        match self {
            Veb::Base(base) => base.delete(x),
            Veb::Rec(rec) => rec.delete(x),
        }
    }
    pub fn collect_vec(&self) -> Vec<usize> {
        if let Some(mut x) = self.min() {
            let mut vec = vec![x];
            while let Some(y) = self.succ(x) {
                vec.push(y);
                x = y;
            }
            vec
        } else {
            Vec::new()
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    len: usize,
    bit: u64,
}
impl Base {
    pub fn new(lg: u32) -> Self {
        Self {
            bit: 0,
            len: 1 << lg,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.bit == 0
    }
    pub fn contains(&self, x: usize) -> bool {
        assert!(x < self.len());
        self.bit >> x & 1 == 1
    }
    pub fn min(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.bit.trailing_zeros() as usize)
        }
    }
    pub fn max(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(size_of::<u64>() as usize * 8 - self.bit.leading_zeros() as usize - 1)
        }
    }
    pub fn prev(&self, x: usize) -> Option<usize> {
        let bit = self.bit & ((1 << x) - 1);
        if bit == 0 {
            None
        } else {
            Some(size_of::<u64>() as usize * 8 - bit.leading_zeros() as usize - 1)
        }
    }
    pub fn succ(&self, x: usize) -> Option<usize> {
        if x == size_of::<u64>() as usize * 8 - 1 {
            None
        } else {
            let bit = self.bit & std::u64::MAX << (x + 1);
            if bit == 0 {
                None
            } else {
                Some(bit.trailing_zeros() as usize)
            }
        }
    }
    pub fn insert(&mut self, x: usize) {
        self.bit |= 1 << x;
    }
    pub fn delete(&mut self, x: usize) -> bool {
        let res = self.contains(x);
        if res {
            self.bit ^= 1 << x;
        }
        res
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Rec {
    lg: u32,
    lower: u32,
    minmax: Option<(usize, usize)>,
    summary: Box<Veb>,
    cluster: HashMap<usize, Veb>,
}
impl Rec {
    pub fn is_empty(&self) -> bool {
        self.minmax.is_none()
    }
    pub fn len(&self) -> usize {
        1 << self.lg
    }
    pub fn new(lg: u32) -> Self {
        assert!(1 < lg);
        let lower = lg / 2;
        let upper = lg - lower;
        Self {
            lg,
            lower,
            minmax: None,
            summary: Box::new(Veb::new(upper)),
            cluster: HashMap::new(),
        }
    }
    pub fn contains(&self, x: usize) -> bool {
        assert!(x < self.len());
        if let Some((min, max)) = self.minmax {
            if min == x || max == x {
                return true;
            }
        }
        let (high, low) = decompose(x, self.lower);
        self.cluster
            .get(&high)
            .map_or(false, |cluster| cluster.contains(low))
    }
    pub fn min(&self) -> Option<usize> {
        self.minmax.map(|(min, _)| min)
    }
    pub fn max(&self) -> Option<usize> {
        self.minmax.map(|(_, max)| max)
    }
    // O (lg lg u)
    // フォールバックせずに O(1) で「どちらを見るか」がわかるのではやいです。
    pub fn prev(&self, x: usize) -> Option<usize> {
        let (min, max) = self.minmax?;
        if max < x {
            Some(max)
        } else {
            let (high, low) = decompose(x, self.lower);
            if self
                .cluster
                .get(&high)
                .map_or(false, |cluster| cluster.min().map_or(false, |y| y < low))
            {
                let low = self.cluster.get(&high).unwrap().prev(low).unwrap();
                Some(index(high, low, self.lower))
            } else if let Some(high) = self.summary.prev(high) {
                let low = self.cluster.get(&high).unwrap().max().unwrap();
                Some(index(high, low, self.lower))
            } else if min < x {
                Some(min)
            } else {
                None
            }
        }
    }
    // O (lg lg u)
    // フォールバックせずに O(1) で「どちらを見るか」がわかるのではやいです。
    pub fn succ(&self, x: usize) -> Option<usize> {
        let (min, max) = self.minmax?;
        if x < min {
            Some(min)
        } else {
            let (high, low) = decompose(x, self.lower);
            if self
                .cluster
                .get(&high)
                .map_or(false, |cluster| cluster.max().map_or(false, |y| low < y))
            {
                let low = self.cluster.get(&high).unwrap().succ(low).unwrap();
                Some(index(high, low, self.lower))
            } else if let Some(high) = self.summary.succ(high) {
                let low = self.cluster.get(&high).unwrap().min().unwrap();
                Some(index(high, low, self.lower))
            } else if x < max {
                Some(max)
            } else {
                None
            }
        }
    }
    // O (lg lg u)
    // サマリーの更新が決して再帰しないのではやいです。
    pub fn insert(&mut self, x: usize) {
        if let Some((min, max)) = self.minmax.as_mut() {
            if min == max {
                if x < *min {
                    *min = x;
                }
                if *max < x {
                    *max = x;
                }
            } else if *min != x && *max != x {
                let mut x = x;
                if x < *min {
                    swap(&mut x, min);
                }
                if *max < x {
                    swap(max, &mut x);
                }
                let (high, low) = decompose(x, self.lower);
                if self
                    .cluster
                    .entry(high)
                    .or_insert(Veb::new(self.lower))
                    .is_empty()
                {
                    self.summary.insert(high);
                }
                self.cluster.get_mut(&high).unwrap().insert(low);
            }
        } else {
            self.minmax = Some((x, x));
        }
    }
    pub fn delete(&mut self, x: usize) -> bool {
        if let Some((min, max)) = self.minmax.as_mut() {
            if min == max {
                if *min == x {
                    self.minmax = None;
                    true
                } else {
                    false
                }
            } else {
                let mut x = x;
                if *min == x {
                    if let Some(high) = self.summary.min() {
                        let low = self.cluster.get(&high).unwrap().min().unwrap();
                        x = index(high, low, self.lower);
                        *min = x;
                    } else {
                        *min = *max;
                        return true;
                    }
                } else if *max == x {
                    if let Some(high) = self.summary.max() {
                        let low = self.cluster.get(&high).unwrap().max().unwrap();
                        x = index(high, low, self.lower);
                        *max = x;
                    } else {
                        *max = *min;
                        return true;
                    }
                }
                let (high, low) = decompose(x, self.lower);
                if self
                    .cluster
                    .entry(high)
                    .or_insert(Veb::new(self.lower))
                    .delete(low)
                {
                    if self.cluster.get(&high).unwrap().is_empty() {
                        let res = self.summary.delete(high);
                        assert!(res);
                        self.cluster.remove(&high);
                    }
                    true
                } else {
                    false
                }
            }
        } else {
            false
        }
    }
}
fn index(high: usize, low: usize, lower: u32) -> usize {
    (high << lower) + low
}
fn decompose(x: usize, lower: u32) -> (usize, usize) {
    (
        x >> lower,
        x & (std::usize::MAX >> (size_of::<usize>() as u32 * 8 - lower)),
    )
}

#[cfg(test)]
mod test {
    use {
        super::Veb,
        rand::prelude::*,
        std::{collections::BTreeSet, time::Instant},
        yansi::Paint,
    };

    #[test]
    fn test_decompose() {
        assert_eq!(super::decompose(10, 2), (2, 2));
    }

    #[test]
    fn test_insert() {
        let mut test = Test::new(3);
        test.insert(2);
        test.insert(3);
        test.insert(4);
    }

    #[test]
    fn test_rand() {
        let mut rng = StdRng::seed_from_u64(42);
        for lg in 3..18 {
            let mut test = Test::new(lg);
            let len = 1 << lg;
            for _ in 0..100 {
                match rng.gen_range(0, 7) {
                    0 => test.contains(rng.gen_range(0, len)),
                    1 => test.min(),
                    2 => test.max(),
                    3 => test.prev(rng.gen_range(0, len)),
                    4 => test.succ(rng.gen_range(0, len)),
                    5 => test.insert(rng.gen_range(0, len)),
                    6 => test.delete(rng.gen_range(0, len)),
                    _ => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn test_speed_veb() {
        let mut rng = StdRng::seed_from_u64(42);
        let lg = 32;
        let start = Instant::now();
        let mut veb = Veb::new(lg);
        let end = Instant::now();
        println!("Construction: {:?}", end - start);

        let len = 1 << lg;
        let q = 1_000_000;
        let start = Instant::now();
        for _ in 0..q {
            match rng.gen_range(0, 7) {
                0 => {
                    veb.contains(rng.gen_range(0, len));
                }
                1 => {
                    veb.min();
                }
                2 => {
                    veb.max();
                }
                3 => {
                    veb.prev(rng.gen_range(0, len));
                }
                4 => {
                    veb.succ(rng.gen_range(0, len));
                }
                5 => {
                    veb.insert(rng.gen_range(0, len));
                }
                6 => {
                    veb.delete(rng.gen_range(0, len));
                }
                _ => unreachable!(),
            }
        }
        let end = Instant::now();
        println!("{} Queries: {:?}", q, end - start);
    }

    #[test]
    fn test_speed_btree() {
        let mut rng = StdRng::seed_from_u64(42);
        let lg = 32;
        let start = Instant::now();
        let mut bts = BTreeSet::new();
        let end = Instant::now();
        println!("Construction: {:?}", end - start);

        let len = 1 << lg;
        let q = 1_000_000;
        let start = Instant::now();
        for _ in 0..q {
            match rng.gen_range(0, 7) {
                0 => {
                    bts.contains(&rng.gen_range(0, len));
                }
                1 => {
                    bts.iter().next();
                }
                2 => {
                    bts.iter().rev().next();
                }
                3 => {
                    bts.range(..rng.gen_range(0, len)).rev().next();
                }
                4 => {
                    bts.range(rng.gen_range(0, len) + 1..).next();
                }
                5 => {
                    bts.insert(rng.gen_range(0, len));
                }
                6 => {
                    bts.remove(&rng.gen_range(0, len));
                }
                _ => unreachable!(),
            }
        }
        let end = Instant::now();
        println!("{} Queries: {:?}", q, end - start);
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
            println!("{}: {:?}", Paint::yellow("Contains").bold(), x);
            assert_eq!(self.veb.contains(x), self.set.contains(&x));
            self.postproces();
        }
        fn min(&self) {
            println!("{}", Paint::magenta("Min").bold());
            assert_eq!(self.veb.min(), self.set.iter().next().copied());
            self.postproces();
        }
        fn max(&self) {
            println!("{}", Paint::magenta("Max").bold());
            assert_eq!(self.veb.max(), self.set.iter().rev().next().copied());
            self.postproces();
        }
        fn prev(&self, x: usize) {
            println!("{}: {:?}", Paint::yellow("Prev").bold(), x);
            assert_eq!(self.veb.prev(x), self.set.range(..x).rev().next().copied());
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
        fn delete(&mut self, x: usize) {
            println!("{}: {:?}", Paint::cyan("Delete").bold(), x);
            let result = self.veb.delete(x);
            let expected = self.set.remove(&x);
            assert_eq!(result, expected);
            self.postproces();
        }
        fn postproces(&self) {
            println!("set = {:?}", &self.set);
            let result = self.veb.collect_vec();
            let expected = self.set.iter().copied().collect::<Vec<_>>();
            assert_eq!(&result, &expected);
        }
    }
}
