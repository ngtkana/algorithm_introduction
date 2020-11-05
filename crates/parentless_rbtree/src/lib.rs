mod paren;

use std::{cmp::Ordering, fmt::Debug, mem::replace};
use yansi::Paint;

pub struct RBTree<K, V>(BoxedNode<K, V>);
impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    pub fn new() -> Self {
        Self(BoxedNode::nil())
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v);
    }
    pub fn remove(&mut self, k: K) -> bool {
        self.0.remove(k).is_some()
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

    // -- me and children
    fn replace(&mut self, x: Self) -> BoxedNode<K, V> {
        replace(self, x)
    }
    fn take(&mut self) -> BoxedNode<K, V> {
        self.replace(Self::nil())
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
        replace(self, child)
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
    fn rotate(self, i: usize) -> Self {
        let mut x = self;
        let mut y = x.take_child(i);
        let z = y.take_child(1 - i);
        x.replace_empty_child(i, z);
        y.replace_empty_child(1 - i, x);
        y
    }

    // -- rb algorithms
    fn insert(&mut self, k: K, v: V) {
        match &mut *self.0 {
            Node::Nil => {
                *self = Self::new(k, v);
            }
            Node::Internal(ref mut internal) => {
                internal.child[if k <= internal.key { 0 } else { 1 }].insert(k, v);
            }
        }
    }
    fn remove(&mut self, k: K) -> Option<Self> {
        let internal = self.as_internal_mut()?;
        match k.cmp(&internal.key) {
            Ordering::Equal => Some(if let Some(mut next) = internal.child[1].remove_first() {
                next.replace_empty_child(0, self.take_child(0));
                next.replace_empty_child(1, self.take_child(1));
                self.replace(next)
            } else {
                self.replace_with_my_child(0)
            }),
            Ordering::Less => internal.child[0].remove(k),
            Ordering::Greater => internal.child[1].remove(k),
        }
        .map(|x| x.assert_isolated())
    }
    fn remove_first(&mut self) -> Option<Self> {
        Some(
            self.as_internal_mut()?.child[0]
                .remove_first()
                .unwrap_or_else(|| self.take())
                .assert_isolated(),
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

#[cfg(test)]
mod tests {
    use super::RBTree;
    use rand::prelude::*;
    use span::Span;

    fn print(rbt: &RBTree<u32, ()>) {
        println!("rbt = {:?}", &rbt);
    }

    fn validate(rbt: &RBTree<u32, ()>, vec: &[u32]) {
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
        validate(rbt, vec);
        println!();
    }

    fn remove(k: u32, rbt: &mut RBTree<u32, ()>, vec: &mut Vec<u32>) {
        println!("Remove {:?}.", &k);
        rbt.remove(k);
        let lb = vec.lower_bound(&k);
        if vec.get(lb).map_or(false, |x| x == &k) {
            vec.remove(lb);
        }
        print(rbt);
        validate(rbt, vec);
        println!();
    }

    #[test]
    fn test_hand() {
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

        let before = x;
        println!("Before: {:?}", &before);
        let after = before.rotate(0);
        println!("After: {:?}", &after);
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

        let before = x;
        println!("Before: {:?}", &before);
        let after = before.rotate(1);
        println!("After: {:?}", &after);
    }

    #[test]
    fn test_rand() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let mut rbt = RBTree::new();
            let mut vec = Vec::new();
            for _ in 0..200 {
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
}