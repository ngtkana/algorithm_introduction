mod paren;
pub mod validate;

use rand::prelude::*;
use std::{cmp::Ordering, fmt::Debug, mem};

pub struct Treap<K, V, R>(BoxedNode<K, V>, R);
impl<K: Ord + Debug, V: Debug, R: Rng> Treap<K, V, R> {
    pub fn new(rng: R) -> Self {
        Self(BoxedNode::nil(), rng)
    }
    pub fn insert(&mut self, k: K, v: V) {
        let node = Node::new(k, v, self.1.next_u64());
        self.0.insert(node);
    }
    pub fn delete(&mut self, k: &K) -> Option<(K, V)> {
        self.0.delete(k).0.map(|node| (node.key, node.value))
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
    fn nil() -> Self {
        Self(None)
    }
    fn is_nil(&self) -> bool {
        self.0.is_none()
    }
    fn new(node: Node<K, V>) -> Self {
        Self(Some(Box::new(node)))
    }
    fn insert(&mut self, node: Node<K, V>) {
        if let Some(internal) = self.0.as_mut() {
            internal.child[if node.key <= internal.key { 0 } else { 1 }].insert(node);
            self.fixup();
        } else {
            *self = Self::new(node);
        }
    }
    fn delete(&mut self, k: &K) -> Self {
        if let Some(internal) = self.0.as_mut() {
            let i = match k.cmp(&internal.key) {
                Ordering::Less => 0,
                Ordering::Greater => 1,
                Ordering::Equal => {
                    return if self.child(1).0.is_some() {
                        let mut rem = self.child_mut(1).delete_first();
                        rem.assert_isolated();
                        (0..2).for_each(|i| rem.replace_empty_child(i, self.take_child(i)));
                        mem::swap(&mut self.unwrap_mut().pri, &mut rem.unwrap_mut().pri);
                        rem.fixup();
                        mem::replace(self, rem)
                    } else {
                        self.replace_by_child(0)
                    };
                }
            };
            let rem = self.child_mut(i).delete(k);
            self.fixup();
            rem
        } else {
            Self::nil()
        }
    }
    fn delete_first(&mut self) -> Self {
        let internal = self.unwrap_mut();
        if internal.child[0].0.is_some() {
            let rem = internal.child[0].delete_first();
            self.fixup();
            rem
        } else {
            self.replace_by_child(1)
        }
    }
    fn fixup(&mut self) {
        if let Some(i) = self.unwrap().child.iter().position(|child| {
            child
                .0
                .as_ref()
                .map_or(false, |child| self.unwrap().pri > child.pri)
        }) {
            self.rotate(i)
        }
    }
    fn collect_vec(&self, vec: &mut Vec<(K, V)>)
    where
        K: Clone,
        V: Clone,
    {
        if let Some(internal) = self.0.as_ref() {
            internal.child[0].collect_vec(vec);
            vec.push((internal.key.clone(), internal.value.clone()));
            internal.child[1].collect_vec(vec);
        }
    }
    fn take(&mut self) -> Self {
        mem::replace(self, Self::nil())
    }

    // -- unwrap
    fn unwrap(&self) -> &Node<K, V> {
        self.0.as_ref().unwrap()
    }
    fn unwrap_mut(&mut self) -> &mut Node<K, V> {
        self.0.as_mut().unwrap()
    }
    fn take_child(&mut self, i: usize) -> Self {
        self.unwrap_mut().child[i].take()
    }
    fn child(&self, i: usize) -> &Self {
        &self.unwrap().child[i]
    }
    fn child_mut(&mut self, i: usize) -> &mut Self {
        &mut self.unwrap_mut().child[i]
    }
    fn assert_isolated(&self) {
        assert!(self.child(0).is_nil());
        assert!(self.child(1).is_nil());
    }
    fn replace_empty_child(&mut self, i: usize, x: BoxedNode<K, V>) {
        let old = mem::replace(&mut self.unwrap_mut().child[i], x);
        assert!(old.is_nil());
    }
    fn replace_by_child(&mut self, i: usize) -> Self {
        assert!(self.child(1 - i).is_nil());
        let x = self.take_child(i);
        mem::replace(self, x)
    }
    fn rotate(&mut self, i: usize) {
        let mut x = self.take();
        let mut y = x.take_child(i);
        let z = y.take_child(1 - i);
        x.replace_empty_child(i, z);
        y.replace_empty_child(1 - i, x);
        *self = y;
    }
}
struct Node<K, V> {
    child: [BoxedNode<K, V>; 2],
    key: K,
    value: V,
    pri: u64,
}
impl<K: Ord + Debug, V: Debug> Node<K, V> {
    fn new(k: K, v: V, p: u64) -> Self {
        Self {
            child: [BoxedNode::nil(), BoxedNode::nil()],
            key: k,
            value: v,
            pri: p,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{validate, Treap};
    use rand::prelude::*;

    #[test]
    fn test_hand() {
        let mut test = Test::seed_from_u64(42);
        test.insert(12);
        test.insert(15);
        test.insert(10);
        test.insert(10);
        test.delete(12);
        test.delete(10);
    }

    #[test]
    fn test_rand_small() {
        test_rand(10, 50, 42);
        test_rand(10, 50, 91);
    }

    #[test]
    fn test_rand_large() {
        test_rand(10, 200, 42);
        test_rand(10, 200, 91);
    }

    #[test]
    fn test_oneline_forward() {
        let mut test = Test::seed_from_u64(42);
        (0..100).for_each(|i| test.insert(i));
        (0..100).for_each(|i| test.delete(i));
    }

    #[test]
    fn test_oneline_backward() {
        let mut test = Test::seed_from_u64(42);
        (0..100).for_each(|i| test.insert(i));
        (0..100).rev().for_each(|i| test.delete(i));
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut test = Test::seed_from_u64(42);
            for _ in 0..q {
                match rng.gen_range(0, 2) {
                    0 => test.insert(rng.gen_range(0, 30)),
                    1 => test.delete(rng.gen_range(0, 30)),
                    _ => panic!(),
                }
            }
        }
    }

    struct Test {
        treap: Treap<u32, (), StdRng>,
        vec: Vec<u32>,
    }
    impl Test {
        fn seed_from_u64(seed: u64) -> Self {
            Self {
                treap: Treap::new(StdRng::seed_from_u64(seed)),
                vec: Vec::new(),
            }
        }
        fn insert(&mut self, x: u32) {
            println!("Insert {:?}", &x);
            self.treap.insert(x, ());
            println!("treap = {:?}", &self.treap);
            let i = self.vec.binary_search(&x).map_or_else(|e| e, |x| x);
            self.vec.insert(i, x);
            self.postprocess();
        }
        fn delete(&mut self, x: u32) {
            println!("Delete {:?}", &x);
            self.treap.delete(&x);
            println!("treap = {:?}", &self.treap);
            if let Ok(i) = self.vec.binary_search(&x) {
                self.vec.remove(i);
            }
            self.postprocess();
        }
        fn postprocess(&self) {
            validate::all(&self.treap);
            assert_eq!(
                &self
                    .treap
                    .collect_vec()
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>(),
                &self.vec
            );
        }
    }
}
