pub mod paren;
pub mod validate;

use std::{
    cmp::Ordering,
    fmt::Debug,
    mem,
    ops::{Deref, DerefMut},
};

pub struct AVLTree<K, V>(BoxedNode<K, V>);
impl<K: Ord + Debug, V: Debug> AVLTree<K, V> {
    pub fn new() -> Self {
        Self(BoxedNode::nil())
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v)
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
    fn insert(&mut self, k: K, v: V) {
        if let Some(internal) = self.0.as_mut() {
            internal.child[if k <= internal.key { 0 } else { 1 }].insert(k, v);
            self.update_balance();
        } else {
            *self = Self(Some(Box::new(Node::new(k, v))));
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
                        rem.update_balance();
                        mem::replace(self, rem)
                    } else {
                        self.replace_by_child(0)
                    };
                }
            };
            let rem = self.child_mut(i).delete(k);
            self.update_balance();
            rem
        } else {
            Self::nil()
        }
    }
    fn delete_first(&mut self) -> Self {
        let internal = self.unwrap_mut();
        if internal.child[0].0.is_some() {
            let rem = self.child_mut(0).delete_first();
            rem.assert_isolated();
            self.update_balance();
            rem
        } else {
            self.replace_by_child(1)
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
    fn ht(&self) -> u32 {
        self.0.as_ref().map_or(0, |x| x.ht)
    }

    // -- unwrap
    fn unwrap(&self) -> &Node<K, V> {
        self.0.as_ref().unwrap().deref()
    }
    fn unwrap_mut(&mut self) -> &mut Node<K, V> {
        self.0.as_mut().unwrap().deref_mut()
    }
    fn update_balance(&mut self) {
        self.update();
        self.balance();
    }
    fn update(&mut self) {
        self.unwrap_mut().ht = self
            .unwrap()
            .child
            .iter()
            .map(|child| child.ht())
            .max()
            .unwrap()
            + 1;
    }
    fn balance(&mut self) {
        for i in 0..2 {
            if self.child(i).ht() == self.child(1 - i).ht() + 2 {
                if self.child(i).child(i).ht() + 1 == self.child(i).child(1 - i).ht() {
                    self.child_mut(i).rotate(1 - i);
                }
                self.rotate(i);
            }
        }
    }
    fn rotate(&mut self, i: usize) {
        let mut x = self.take();
        let mut y = x.take_child(i);
        let z = y.take_child(1 - i);
        x.replace_empty_child(i, z);
        x.update();
        y.replace_empty_child(1 - i, x);
        y.update();
        *self = y;
    }
    fn take(&mut self) -> Self {
        mem::replace(self, Self::nil())
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
    fn take_child(&mut self, i: usize) -> BoxedNode<K, V> {
        self.unwrap_mut().child[i].take()
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
}

pub struct Node<K, V> {
    child: [BoxedNode<K, V>; 2],
    ht: u32,
    key: K,
    value: V,
}
impl<K: Ord + Debug, V: Debug> Node<K, V> {
    fn new(k: K, v: V) -> Self {
        Node {
            child: [BoxedNode::nil(), BoxedNode::nil()],
            ht: 1,
            key: k,
            value: v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{validate, AVLTree};
    use rand::prelude::*;

    #[test]
    fn test_hand() {
        let mut test = Test::new();
        test.insert(10);
        test.insert(11);
        test.insert(15);
        test.insert(17);
        test.delete(17);
        test.delete(11);
        test.delete(12);
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
    fn test_oneline() {
        let mut test = Test::new();
        (0..100).for_each(|i| test.insert(i));
        (0..100).for_each(|i| test.delete(i));
    }

    #[test]
    fn test_oneline_reverse() {
        let mut test = Test::new();
        (0..100).for_each(|i| test.insert(i));
        (0..100).rev().for_each(|i| test.delete(i));
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut test = Test::new();
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
        avl: AVLTree<u32, ()>,
        vec: Vec<u32>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                avl: AVLTree::new(),
                vec: Vec::new(),
            }
        }
        fn insert(&mut self, x: u32) {
            println!("Insert {:?}", &x);
            self.avl.insert(x, ());
            println!("alv = {:?}", &self.avl);
            let i = self.vec.binary_search(&x).map_or_else(|e| e, |x| x);
            self.vec.insert(i, x);
            self.postprocess();
        }
        fn delete(&mut self, x: u32) {
            println!("Delete {:?}", &x);
            self.avl.delete(&x);
            println!("alv = {:?}", &self.avl);
            if let Ok(i) = self.vec.binary_search(&x) {
                self.vec.remove(i);
            }
            self.postprocess();
        }
        fn postprocess(&self) {
            validate::all(&self.avl);
            assert_eq!(
                &self
                    .avl
                    .collect_vec()
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>(),
                &self.vec
            );
        }
    }
}
