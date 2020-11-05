mod paren;
pub mod validate;

use std::{cmp::Ordering, fmt::Debug, mem::replace};
use yansi::Paint;

pub struct RBTree<K, V>(BoxedNode<K, V>);
impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    pub fn new() -> Self {
        Self(BoxedNode::nil())
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v);
        self.0.set_color(Color::Black);
    }
    pub fn remove(&mut self, k: K) -> bool {
        let res = self.0.remove(k).is_some();
        self.0.set_color(Color::Black);
        res
    }
    pub fn collect(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut vec = Vec::new();
        self.0.collect(&mut vec);
        vec
    }
}

struct BoxedNode<K, V>(Box<Node<K, V>>);
impl<K: Ord + Debug, V: Debug> BoxedNode<K, V> {
    fn nil() -> Self {
        Self(Box::new(Node::Nil))
    }
    fn is_nil(&self) -> bool {
        matches!(*self.0, Node::Nil)
    }
    fn new(k: K, v: V) -> Self {
        Self(Box::new(Node::Internal(Internal {
            child: [Self::nil(), Self::nil()],
            key: k,
            value: v,
            color: Color::Red,
        })))
    }
    fn as_internal(&self) -> Option<&Internal<K, V>> {
        match &*self.0 {
            Node::Nil => None,
            Node::Internal(internal) => Some(internal),
        }
    }
    fn as_internal_mut(&mut self) -> Option<&mut Internal<K, V>> {
        match &mut *self.0 {
            Node::Nil => None,
            Node::Internal(internal) => Some(internal),
        }
    }

    // -- color
    fn color(&self) -> Color {
        self.as_internal().map_or(Color::Black, |x| x.color)
    }
    fn is_red(&self) -> bool {
        self.color() == Color::Red
    }
    fn is_black(&self) -> bool {
        self.color() == Color::Black
    }
    fn assert_red(&self) -> &Self {
        assert!(self.is_red());
        self
    }
    fn assert_black(&self) -> &Self {
        assert!(self.is_black());
        self
    }
    fn set_color(&mut self, color: Color) {
        let internal = self.as_internal_mut().unwrap();
        internal.color = color;
    }
    fn swap_color(&mut self, other: &mut Self) {
        let x = self.as_internal_mut().unwrap();
        let y = other.as_internal_mut().unwrap();
        std::mem::swap(&mut x.color, &mut y.color);
    }
    fn swap_color_with_child(&mut self, i: usize) {
        let l = self.color();
        let r = self.child(i).color();
        self.set_color(r);
        self.child_mut(i).set_color(l);
    }

    // -- me and children
    fn replace(&mut self, x: Self) -> BoxedNode<K, V> {
        std::mem::replace(self, x)
    }
    fn take(&mut self) -> BoxedNode<K, V> {
        self.replace(Self::nil())
    }
    fn child(&self, i: usize) -> &BoxedNode<K, V> {
        &self.as_internal().unwrap().child[i]
    }
    fn child_mut(&mut self, i: usize) -> &mut BoxedNode<K, V> {
        &mut self.as_internal_mut().unwrap().child[i]
    }
    fn replace_empty_child(&mut self, i: usize, x: Self) {
        let internal = self.as_internal_mut().unwrap();
        let old = internal.child[i].replace(x);
        assert!(old.is_nil());
    }
    fn replace_child(&mut self, i: usize, x: Self) -> BoxedNode<K, V> {
        let internal = self.as_internal_mut().unwrap();
        internal.child[i].replace(x)
    }
    fn take_child(&mut self, i: usize) -> BoxedNode<K, V> {
        self.replace_child(i, Self::nil())
    }
    fn replace_with_my_child(&mut self, i: usize) -> BoxedNode<K, V> {
        let internal = self.as_internal_mut().unwrap();
        assert!(internal.child[1 - i].is_nil());
        let child = internal.child[i].take();
        replace(self, child).assert_isolated()
    }

    // -- assertions
    fn assert_isolated(self) -> Self {
        let internal = self.as_internal().unwrap();
        assert!(internal.child[0].is_nil());
        assert!(internal.child[1].is_nil());
        self
    }

    // -- collect
    fn collect(&self, vec: &mut Vec<(K, V)>)
    where
        K: Clone,
        V: Clone,
    {
        if let Some(internal) = self.as_internal() {
            internal.child[0].collect(vec);
            vec.push((internal.key.clone(), internal.value.clone()));
            internal.child[1].collect(vec);
        }
    }

    // -- deformations
    fn rotate(&mut self, i: usize) {
        let mut x = self.take();
        let mut y = x.take_child(i);
        let z = y.take_child(1 - i);
        x.replace_empty_child(i, z);
        y.replace_empty_child(1 - i, x);
        *self = y;
    }
    fn swap_color_rotate(&mut self, i: usize) {
        self.swap_color_with_child(i);
        self.rotate(i);
    }

    // -- rb algorithms
    fn insert(&mut self, k: K, v: V) -> Option<DoubleRed> {
        match &mut *self.0 {
            Node::Nil => {
                *self = Self::new(k, v);
                Some(DoubleRed::Me)
            }
            Node::Internal(ref mut internal) => {
                let i = if k <= internal.key { 0 } else { 1 };
                match internal.child[i].insert(k, v)? {
                    DoubleRed::Me => match self.color() {
                        Color::Red => Some(DoubleRed::Child(i)),
                        Color::Black => None,
                    },
                    DoubleRed::Child(j) => self.insert_fixup(i, j),
                }
            }
        }
    }
    fn insert_fixup(&mut self, i: usize, j: usize) -> Option<DoubleRed> {
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
    fn remove(&mut self, k: K) -> Option<(Self, Option<Charge>)> {
        let internal = self.as_internal_mut()?;
        let i = match k.cmp(&internal.key) {
            Ordering::Equal => {
                return Some(
                    if let Some((mut next, e)) = internal.child[1].remove_first() {
                        self.swap_color(&mut next);
                        next.replace_empty_child(0, self.take_child(0));
                        next.replace_empty_child(1, self.take_child(1));
                        (self.replace(next), e.and_then(|_| self.remove_fixup(1)))
                    } else {
                        let removed = self.replace_with_my_child(0).assert_isolated();
                        let charged = removed.is_black();
                        (removed, if charged { Some(Charge()) } else { None })
                    },
                )
            }
            Ordering::Less => 0,
            Ordering::Greater => 1,
        };
        let (removed, charge) = internal.child[i].remove(k)?;
        Some((removed, charge.and_then(|_| self.remove_fixup(i))))
    }
    fn remove_fixup(&mut self, i: usize) -> Option<Charge> {
        match self.child(i).color() {
            Color::Red => {
                self.child_mut(i).set_color(Color::Black);
                None
            }
            Color::Black => match self.child(1 - i).color() {
                Color::Red => {
                    self.swap_color_rotate(1 - i);
                    self.child_mut(i)
                        .remove_fixup(i)
                        .and_then(|Charge()| self.remove_fixup(i))
                }
                Color::Black => match (
                    self.child(1 - i).child(i).color(),
                    self.child(1 - i).child(1 - i).color(),
                ) {
                    (Color::Black, Color::Black) => {
                        self.child_mut(1 - i).set_color(Color::Red);
                        Some(Charge())
                    }
                    (Color::Red, Color::Black) => {
                        self.child_mut(1 - i).swap_color_rotate(i);
                        self.remove_fixup(i)
                    }
                    (_, Color::Red) => {
                        self.child_mut(1 - i)
                            .child_mut(1 - i)
                            .set_color(Color::Black);
                        self.swap_color_rotate(1 - i);
                        None
                    }
                },
            },
        }
    }
    fn remove_first(&mut self) -> Option<(Self, Option<Charge>)> {
        Some(
            if let Some((x, e)) = self.as_internal_mut()?.child[0].remove_first() {
                (x, e.and_then(|Charge()| self.remove_fixup(0)))
            } else {
                let removed = self.replace_with_my_child(1).assert_isolated();
                let charged = removed.is_black();
                (removed, if charged { Some(Charge()) } else { None })
            },
        )
    }
}
enum Node<K, V> {
    Internal(Internal<K, V>),
    Nil,
}
struct Internal<K, V> {
    child: [BoxedNode<K, V>; 2],
    key: K,
    value: V,
    color: Color,
}
#[derive(Clone, PartialEq, Copy)]
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
struct Charge();

#[cfg(test)]
mod tests {
    use super::{validate, RBTree};
    use rand::prelude::*;
    use span::Span;

    #[test]
    fn test_hand_insert_delete() {
        let mut rbt = RBTree::new();
        let mut vec = Vec::new();

        insert(10, &mut rbt, &mut vec);
        insert(12, &mut rbt, &mut vec);
        insert(8, &mut rbt, &mut vec);
        insert(7, &mut rbt, &mut vec);
        insert(9, &mut rbt, &mut vec);
        insert(11, &mut rbt, &mut vec);
        insert(13, &mut rbt, &mut vec);

        remove(10, &mut rbt, &mut vec);
        remove(12, &mut rbt, &mut vec);
        remove(13, &mut rbt, &mut vec);
    }

    #[test]
    fn test_hand_rotation_left() {
        use super::BoxedNode;
        let l = BoxedNode::new(0, ());
        let z = BoxedNode::new(2, ());
        let r = BoxedNode::new(4, ());
        let mut y = BoxedNode::new(1, ());
        y.replace_empty_child(0, l);
        y.replace_empty_child(1, z);
        let mut x = BoxedNode::new(3, ());
        x.replace_empty_child(0, y);
        x.replace_empty_child(1, r);

        println!("Before: {:?}", &x);
        x.rotate(0);
        println!("After: {:?}", &x);
    }

    #[test]
    fn test_exhaustive_insert() {
        use next_permutation::next_permutation;
        let n = 7;
        let mut perm = (0..n).collect::<Vec<u32>>();
        while {
            let mut rbt = RBTree::new();
            let mut vec = Vec::new();
            for &x in &perm {
                insert(x, &mut rbt, &mut vec);
            }
            next_permutation(&mut perm)
        } {}
    }

    #[test]
    fn test_hand_rotation_right() {
        use super::BoxedNode;
        let l = BoxedNode::new(0, ());
        let z = BoxedNode::new(2, ());
        let r = BoxedNode::new(4, ());
        let mut y = BoxedNode::new(3, ());
        y.replace_empty_child(0, z);
        y.replace_empty_child(1, r);
        let mut x = BoxedNode::new(1, ());
        x.replace_empty_child(0, l);
        x.replace_empty_child(1, y);

        println!("Before: {:?}", &x);
        x.rotate(1);
        println!("After: {:?}", &x);
    }

    #[test]
    fn test_rand_small() {
        test_rand(2000, 20, 42);
    }

    #[test]
    fn test_rand_large() {
        test_rand(20, 100, 42);
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut rbt = RBTree::new();
            let mut vec = Vec::new();
            for _ in 0..q {
                match rng.gen_range(0, 2) {
                    0 => {
                        let key = rng.gen_range(0, 30);
                        insert(key, &mut rbt, &mut vec);
                    }
                    1 => {
                        let key = rng.gen_range(0, 30);
                        insert(key, &mut rbt, &mut vec);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn print(rbt: &RBTree<u32, ()>) {
        println!("rbt = {:?}", &rbt);
    }
    fn compare(rbt: &RBTree<u32, ()>, vec: &[u32]) {
        assert_eq!(
            rbt.collect()
                .iter()
                .map(|&(k, _)| k)
                .collect::<Vec<_>>()
                .as_slice(),
            vec
        );
    }
    fn insert(k: u32, rbt: &mut RBTree<u32, ()>, vec: &mut Vec<u32>) {
        println!("Insert {:?}.", &k);
        rbt.insert(k, ());
        let lb = vec.lower_bound(&k);
        vec.insert(lb, k);
        print(rbt);
        compare(rbt, vec);
        println!();
        validate::all(rbt);
    }
    fn remove(k: u32, rbt: &mut RBTree<u32, ()>, vec: &mut Vec<u32>) {
        println!("Remove {:?}.", &k);
        rbt.remove(k);
        let lb = vec.lower_bound(&k);
        if vec.get(lb).map_or(false, |x| x == &k) {
            vec.remove(lb);
        }
        print(rbt);
        compare(rbt, vec);
        println!();
        validate::all(rbt);
    }
}
