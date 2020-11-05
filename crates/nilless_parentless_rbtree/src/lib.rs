mod paren;
pub mod validate;

use dbg::msg;
use std::{cmp::Ordering, fmt::Debug, mem::replace};
use yansi::Paint;

pub struct RBTree<K, V>(BoxedNode<K, V>);
impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    pub fn new() -> Self {
        Self(BoxedNode(None))
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v);
        self.0.set_color(Color::Black);
    }
    pub fn delete(&mut self, k: K) -> bool {
        self.0.delete(k).is_some()
    }
    pub fn collect_vec(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut vec = Vec::new();
        self.0.collect_vec(&mut vec);
        vec
    }
}
struct BoxedNode<K, V>(Option<Box<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> BoxedNode<K, V> {
    fn new(k: K, v: V) -> Self {
        Self(Some(Box::new(Node {
            child: [Self(None), Self(None)],
            key: k,
            value: v,
            color: Color::Red,
        })))
    }

    // -- color
    fn color(&self) -> Color {
        self.0.as_ref().map_or(Color::Black, |x| x.color)
    }
    fn is_red(&self) -> bool {
        self.color() == Color::Red
    }
    fn is_black(&self) -> bool {
        self.color() == Color::Black
    }
    fn set_color(&mut self, color: Color) {
        self.0.as_mut().unwrap().color = color;
    }
    fn swap_color_with_child(&mut self, i: usize) {
        let color = self.color();
        self.set_color(self.child(i).color());
        self.child_mut(i).set_color(color);
    }
    fn assert_red(&self) -> &Self {
        assert!(self.is_red());
        self
    }
    fn assert_black(&self) -> &Self {
        assert!(self.is_black());
        self
    }

    // -- me and child
    fn assert_isolated(self) -> Self {
        assert!((0..2).all(|i| self.0.as_ref().unwrap().child[i].0.is_none()));
        self
    }
    fn child(&self, i: usize) -> &Self {
        &self.0.as_ref().unwrap().child[i]
    }
    fn child_mut(&mut self, i: usize) -> &mut Self {
        &mut self.0.as_mut().unwrap().child[i]
    }
    fn replace_child(&mut self, i: usize, x: Self) -> Self {
        replace(&mut self.0.as_mut().unwrap().child[i], x)
    }
    fn take_child(&mut self, i: usize) -> Self {
        Self(self.0.as_mut().unwrap().child[i].0.take())
    }
    fn replace_empty_child(&mut self, i: usize, x: Self) {
        assert!(self.replace_child(i, x).0.is_none());
    }
    fn transplant_child(&mut self, i: usize) -> BoxedNode<K, V> {
        let internal = self.0.as_mut().unwrap();
        assert!(internal.child[1 - i].0.is_none());
        let child = internal.child[i].0.take();
        replace(self, Self(child)).assert_isolated()
    }

    // -- deformatinos
    fn rotate(&mut self, i: usize) {
        let mut x = replace(self, Self(None));
        let mut y = x.take_child(i);
        x.replace_empty_child(i, y.take_child(1 - i));
        y.replace_empty_child(1 - i, x);
        *self = y;
    }
    fn swap_color_rotate(&mut self, i: usize) {
        self.swap_color_with_child(i);
        self.rotate(i);
    }

    // -- rb operations
    fn insert(&mut self, k: K, v: V) -> Option<DoubleRed> {
        if let Some(internal) = &mut self.0 {
            let i = if k <= internal.key { 0 } else { 1 };
            match internal.child[i].insert(k, v)? {
                DoubleRed::Me => match internal.color {
                    Color::Red => Some(DoubleRed::Child(i)),
                    Color::Black => None,
                },
                DoubleRed::Child(j) => self.insert_fixup(i, j),
            }
        } else {
            *self = Self::new(k, v);
            Some(DoubleRed::Me)
        }
    }
    fn delete(&mut self, k: K) -> Option<Box<Node<K, V>>> {
        msg!("delete", &self);
        let internal = self.0.as_mut()?;
        let i = match k.cmp(&internal.key) {
            Ordering::Equal => {
                return if let Some(res) = internal.child[1].delete_first() {
                    let mut res = Self(Some(res));
                    res.replace_empty_child(0, self.take_child(0));
                    res.replace_empty_child(1, self.take_child(1));
                    replace(self, res).assert_isolated().0
                } else {
                    self.transplant_child(0).0
                }
            }
            Ordering::Less => 0,
            Ordering::Greater => 1,
        };
        internal.child[i].delete(k)
    }
    fn insert_fixup(&mut self, i: usize, j: usize) -> Option<DoubleRed> {
        msg!("insert_fixup", (&self, i, j));
        self.assert_black()
            .child(i)
            .assert_red()
            .child(j)
            .assert_red();
        match self.child(1 - i).color() {
            Color::Red => {
                self.set_color(Color::Red);
                self.child_mut(i).set_color(Color::Black);
                self.child_mut(1 - i).set_color(Color::Black);
                Some(DoubleRed::Me)
            }
            Color::Black => {
                if i == j {
                    self.swap_color_rotate(i);
                    None
                } else {
                    self.child_mut(i).rotate(j);
                    self.insert_fixup(i, 1 - j)
                }
            }
        }
    }
    fn delete_first(&mut self) -> Option<Box<Node<K, V>>> {
        msg!("delete_first", &self);
        let res = self.0.as_mut()?.child[0].delete_first();
        res.or_else(|| self.transplant_child(1).0)
    }
    fn collect_vec(&self, vec: &mut Vec<(K, V)>)
    where
        K: Clone,
        V: Clone,
    {
        if let Some(internal) = &self.0 {
            internal.child[0].collect_vec(vec);
            vec.push((internal.key.clone(), internal.value.clone()));
            internal.child[1].collect_vec(vec);
        }
    }
}

struct Node<K, V> {
    child: [BoxedNode<K, V>; 2],
    key: K,
    value: V,
    color: Color,
}
#[derive(Debug, Clone, PartialEq, Copy)]
enum Color {
    Red,
    Black,
}
impl Color {
    fn paint<T>(&self, x: T) -> Paint<T> {
        match self {
            Color::Red => Paint::red(x),
            Color::Black => Paint::blue(x),
        }
        .bold()
    }
}
enum DoubleRed {
    Me,
    Child(usize),
}

#[cfg(test)]
mod tests {
    use super::validate;
    use super::RBTree;
    use rand::prelude::*;

    #[test]
    fn test_hand() {
        let mut test = Test::new();
        test.insert(10);
        test.insert(12);
        test.insert(11);
    }

    #[test]
    fn test_exhaustive_insert() {
        use next_permutation::next_permutation;
        let n = 7;
        let mut perm = (0..n).collect::<Vec<u32>>();
        while {
            let mut test = Test::new();
            perm.iter().for_each(|&x| test.insert(x));
            next_permutation(&mut perm)
        } {}
    }

    #[test]
    fn test_rand_small() {
        test_rand(2000, 20, 42);
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut test = Test::new();
        for _ in 0..t {
            for _ in 0..q {
                match rng.gen_range(0, 2) {
                    0 => test.insert(rng.gen_range(0, 30)),
                    1 => test.delete(rng.gen_range(0, 30)),
                    _ => unreachable!(),
                }
            }
        }
    }

    struct Test {
        rbt: RBTree<u32, ()>,
        vec: Vec<u32>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                rbt: RBTree::new(),
                vec: Vec::new(),
            }
        }
        fn assert_eq(&self) {
            validate::all(&self.rbt);
            println!("Comparing rbt = {:?}", &self.rbt);
            assert_eq!(
                &self
                    .rbt
                    .collect_vec()
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>(),
                &self.vec,
            );
        }
        fn insert(&mut self, k: u32) {
            println!("Insert {:?}", &k);
            self.rbt.insert(k, ());
            let i = self.vec.binary_search(&k).map_or_else(|e| e, |i| i);
            self.vec.insert(i, k);
            self.assert_eq();
        }
        fn delete(&mut self, k: u32) {
            println!("Delete {:?}", &k);
            self.rbt.delete(k);
            if let Ok(i) = self.vec.binary_search(&k) {
                self.vec.remove(i);
            }
            self.assert_eq();
        }
    }
}
