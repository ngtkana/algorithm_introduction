mod color;
mod paren;
mod validate;

use color::Color;
use std::{cmp::Ordering, fmt::Debug};

pub struct RBTree<K>(Option<BoxNode<K>>);
impl<K: Ord + Debug> RBTree<K> {
    pub fn new() -> Self {
        Self(None)
    }
    pub fn from_slice(src: &[K]) -> Self
    where
        K: Clone,
    {
        if src.is_empty() {
            Self(None)
        } else {
            Self(Some(BoxNode::from_slice(src)))
        }
    }
    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |root| root.size())
    }
    pub fn append(&mut self, other: &mut Self) {
        let x = std::mem::replace(self, Self(None));
        let y = std::mem::replace(other, Self(None));
        *self = Self::merge(x, y);
    }
    pub fn merge(x: Self, y: Self) -> Self {
        Self(BoxNode::option_merge(x.0, y.0))
    }
    pub fn split_off(&mut self, k: usize) -> Self {
        let x = std::mem::replace(self, Self(None));
        let (x, y) = x.split(k);
        *self = x;
        y
    }
    pub fn split(self, k: usize) -> (Self, Self) {
        match self.0 {
            None => {
                assert_eq!(k, 0);
                (Self(None), Self(None))
            }
            Some(root) => {
                let (l, r) = root.split(k);
                (Self(l), Self(r))
            }
        }
    }
    pub fn collect_vec(&self) -> Vec<K>
    where
        K: Clone,
    {
        let mut vec = Vec::new();
        self.0
            .as_ref()
            .iter()
            .for_each(|root| root.collect_vec(&mut vec));
        vec
    }
}

pub enum BoxNode<K> {
    Internal(Box<Node<K>>),
    Leaf(Leaf<K>),
}
impl<K: Ord + Debug> BoxNode<K> {
    // -- ctors
    pub fn leaf(k: K) -> Self {
        Self::Leaf(Leaf { key: k })
    }
    pub fn internal(x: Self, y: Self, color: Color) -> Self {
        assert_eq!(
            x.bh_aug(),
            y.bh_aug(),
            "Inconsistent bh_aug: x = {:?}, y = {:?}",
            &x,
            &y
        );
        Self::Internal(Box::new(Node {
            color,
            bh: x.bh_aug(),
            size: x.size() + y.size(),
            child: [x, y],
        }))
    }
    pub fn from_slice(src: &[K]) -> Self
    where
        K: Clone,
    {
        assert!(!src.is_empty());
        let n = src.len();
        if n == 1 {
            Self::leaf(src[0].clone())
        } else {
            let mut res = Self::merge(
                Self::from_slice(&src[..n / 2]),
                Self::from_slice(&src[n / 2..]),
            );
            res.set_color(Color::Black);
            res
        }
    }

    // -- converters
    fn into_node(self) -> Option<Box<Node<K>>> {
        match self {
            Self::Internal(internal) => Some(internal),
            Self::Leaf(_) => None,
        }
    }
    fn as_node(&self) -> Option<&Box<Node<K>>> {
        match self {
            Self::Internal(internal) => Some(internal),
            Self::Leaf(_) => None,
        }
    }
    fn as_node_mut(&mut self) -> Option<&mut Box<Node<K>>> {
        match self {
            Self::Internal(internal) => Some(internal),
            Self::Leaf(_) => None,
        }
    }

    // -- size
    fn size(&self) -> usize {
        self.as_node().map_or(1, |node| node.size)
    }

    // -- color
    fn color(&self) -> Color {
        self.as_node().map_or(Color::Black, |x| x.color)
    }
    fn is_red(&self) -> bool {
        self.color() == Color::Red
    }
    fn is_black(&self) -> bool {
        self.color() == Color::Black
    }
    fn bh(&self) -> u32 {
        self.as_node().map_or(0, |node| node.bh)
    }
    fn bh_aug(&self) -> u32 {
        self.bh()
            + match self.color() {
                Color::Red => 0,
                Color::Black => 1,
            }
    }
    fn color_black(&mut self) {
        self.as_node_mut()
            .into_iter()
            .for_each(|node| node.color = Color::Black);
    }

    // -- unwrap
    fn unwrap(&self) -> &Node<K> {
        self.as_node().unwrap()
    }
    fn unwrap_mut(&mut self) -> &mut Node<K> {
        self.as_node_mut().unwrap()
    }
    fn child(&self, i: usize) -> &Self {
        &self.unwrap().child[i]
    }
    fn set_color(&mut self, color: Color) {
        self.unwrap_mut().color = color
    }
    fn swap_color_rotate_left(self) -> Self {
        let x = self;
        let Node {
            child: [y, x1],
            color,
            bh: _,
            size: _,
        } = *x.into_node().unwrap();
        assert!(y.is_red());
        let Node {
            child: [y0, y1],
            color: _,
            bh: _,
            size: _,
        } = *y.into_node().unwrap();
        let x = Self::internal(y1, x1, Color::Red);
        Self::internal(y0, x, color)
    }
    fn swap_color_rotate_right(self) -> Self {
        let x = self;
        let Node {
            child: [x0, y],
            color,
            bh: _,
            size: _,
        } = *x.into_node().unwrap();
        assert!(y.is_red());
        let Node {
            child: [y0, y1],
            color: _,
            bh: _,
            size: _,
        } = *y.into_node().unwrap();
        let x = Self::internal(x0, y0, Color::Red);
        Self::internal(x, y1, color)
    }

    // -- rb ops
    pub fn option_merge(x: Option<Self>, y: Option<Self>) -> Option<Self> {
        match (x, y) {
            (None, None) => None,
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (Some(x), Some(y)) => Some(BoxNode::merge(x, y)),
        }
    }
    pub fn merge(x: Self, y: Self) -> Self {
        let mut res = Self::merge_impl(x, y);
        res.color_black();
        res
    }
    pub fn merge_impl(mut x: Self, mut y: Self) -> Self {
        let res = match x.bh().cmp(&y.bh()) {
            Ordering::Less => {
                let Node {
                    child: [c, mut y],
                    color,
                    bh: _,
                    size: _,
                } = *y.into_node().unwrap();
                let mut x = Self::merge_impl(x, c);
                if color == Color::Black && x.is_red() && x.child(0).is_red() {
                    match y.color() {
                        Color::Black => {
                            let mut root = Self::internal(x, y, color);
                            root = root.swap_color_rotate_left();
                            root
                        }
                        Color::Red => {
                            x.set_color(Color::Black);
                            y.set_color(Color::Black);
                            Self::internal(x, y, Color::Red)
                        }
                    }
                } else {
                    Self::internal(x, y, color)
                }
            }
            Ordering::Greater => {
                let Node {
                    child: [mut x, c],
                    color,
                    bh: _,
                    size: _,
                } = *x.into_node().unwrap();
                let mut y = Self::merge_impl(c, y);
                if color == Color::Black && y.is_red() && y.child(1).is_red() {
                    match x.color() {
                        Color::Black => {
                            let mut root = Self::internal(x, y, color);
                            root = root.swap_color_rotate_right();
                            root
                        }
                        Color::Red => {
                            x.set_color(Color::Black);
                            y.set_color(Color::Black);
                            Self::internal(x, y, Color::Red)
                        }
                    }
                } else {
                    Self::internal(x, y, color)
                }
            }
            Ordering::Equal => {
                x.color_black();
                y.color_black();
                Self::internal(x, y, Color::Red)
            }
        };
        res
    }
    pub fn split(self, k: usize) -> (Option<Self>, Option<Self>) {
        assert!(k <= self.size());
        if k == 0 {
            (None, Some(self))
        } else if k == self.size() {
            (Some(self), None)
        } else {
            let Node {
                child: [mut l, mut r],
                color: _,
                bh: _,
                size: _,
            } = *self.into_node().unwrap();
            match k.cmp(&l.size()) {
                Ordering::Less => {
                    let (mut l, c) = l.split(k);
                    l.as_mut().into_iter().for_each(|l| l.color_black());
                    (l, Self::option_merge(c, Some(r)))
                }
                Ordering::Greater => {
                    let (c, mut r) = r.split(k - l.size());
                    r.as_mut().into_iter().for_each(|r| r.color_black());
                    (Self::option_merge(Some(l), c), r)
                }
                Ordering::Equal => {
                    l.color_black();
                    r.color_black();
                    (Some(l), Some(r))
                }
            }
        }
    }
    pub fn collect_vec(&self, vec: &mut Vec<K>)
    where
        K: Clone,
    {
        match self {
            Self::Internal(internal) => internal
                .child
                .iter()
                .for_each(|child| child.collect_vec(vec)),
            Self::Leaf(Leaf { key }) => vec.push(key.clone()),
        }
    }
}

pub struct Node<K> {
    child: [BoxNode<K>; 2],
    color: Color,
    bh: u32,
    size: usize,
}
pub struct Leaf<K> {
    key: K,
}

#[cfg(test)]
mod tests {
    use super::{validate, RBTree};
    use rand::prelude::*;
    use std::{collections::LinkedList, iter::once};

    #[test]
    fn test_rand_small() {
        test_rand(20, 200, 10, 42);
        test_rand(20, 200, 10, 43);
        test_rand(20, 200, 10, 44);
    }

    #[test]
    fn test_rand_large() {
        test_rand(2, 200, 100, 42);
        test_rand(2, 200, 100, 43);
        test_rand(2, 200, 100, 44);
    }

    fn test_rand(t: u32, q: u32, n_max: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for ti in 0..t {
            let n = rng.gen_range(1, n_max);
            println!("Init n = {:?}", n);
            let mut test = Test::build(&(0..n).collect::<Vec<_>>());
            for qi in 0..q {
                println!("Test {}, query {}", ti, qi);
                match rng.gen_range(0, 3) {
                    0 | 1 => {
                        if test.len() == 1 {
                            println!("Skipped merge");
                        } else {
                            let i = rng.gen_range(0, test.len() - 1);
                            test.merge(i);
                        }
                    }
                    2 => {
                        let (i, j) = loop {
                            let i = rng.gen_range(0, test.len());
                            let len = test.ith_len(i);
                            if len == 0 {
                                continue;
                            }
                            let j = rng.gen_range(0, test.ith_len(i));
                            break (i, j);
                        };
                        test.split(i, j);
                    }
                    _ => panic!(),
                }
            }
        }
    }

    struct Test {
        len: usize,
        vec: LinkedList<Vec<u32>>,
        rbt: LinkedList<RBTree<u32>>,
    }
    impl Test {
        pub fn build(src: &[u32]) -> Self {
            Self {
                rbt: once(RBTree::from_slice(src)).collect(),
                vec: once(src.to_vec()).collect(),
                len: 1,
            }
        }
        pub fn len(&self) -> usize {
            self.len
        }
        pub fn ith_len(&self, i: usize) -> usize {
            let x = self.vec.iter().nth(i).unwrap().len();
            let y = self.rbt.iter().nth(i).unwrap().len();
            assert_eq!(
                x, y,
                "vec and rbt has different lengths: vec = {:?}, rbt = {:?}",
                &self.vec, &self.rbt
            );
            x
        }
        pub fn merge(&mut self, i: usize) {
            println!("Merge {:?}", i);
            assert!(i < self.len() - 1);

            // -- vec
            let mut tail = self.vec.split_off(i + 2);
            let y = self.vec.pop_back().unwrap();
            self.vec.back_mut().unwrap().extend(y.into_iter());
            self.vec.append(&mut tail);

            // -- rbt
            let mut tail = self.rbt.split_off(i + 2);
            let mut y = self.rbt.pop_back().unwrap();
            self.rbt.back_mut().unwrap().append(&mut y);
            self.rbt.append(&mut tail);

            self.len -= 1;
            self.postprocess();
        }

        pub fn split(&mut self, i: usize, j: usize) {
            println!("Split {:?} {:?}", i, j);
            assert!(i < self.len());

            // -- vec
            let mut tail = self.vec.split_off(i + 1);
            let y = self.vec.back_mut().unwrap().split_off(j);
            self.vec.push_back(y);
            self.vec.append(&mut tail);

            // -- rbt
            let mut tail = self.rbt.split_off(i + 1);
            let y = self.rbt.back_mut().unwrap().split_off(j);
            self.rbt.push_back(y);
            self.rbt.append(&mut tail);

            self.len += 1;
            self.postprocess();
        }
        fn postprocess(&self) {
            println!("rbt = {:?}", &self.rbt);
            self.vec.iter().zip(self.rbt.iter()).for_each(|(vec, rbt)| {
                validate::all(rbt);
                assert_eq!(
                    vec.as_slice(),
                    rbt.collect_vec().as_slice(),
                    "vec = {:?}, rbt = {:?}",
                    &vec,
                    &rbt
                );
            });
        }
    }
}
