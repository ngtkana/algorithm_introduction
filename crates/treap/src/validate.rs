use super::{BoxedNode, Node, Treap};
use rand::prelude::*;
use std::fmt::Debug;

pub fn all<T: Validate>(x: &T) {
    x.heap()
}

pub trait Validate {
    fn heap(&self);
}
impl<K: Ord + Debug, V: Debug, R: Rng> Validate for Treap<K, V, R> {
    fn heap(&self) {
        self.0.heap()
    }
}
impl<K: Ord + Debug, V: Debug> Validate for BoxedNode<K, V> {
    fn heap(&self) {
        self.0.as_ref().iter().for_each(|x| x.heap())
    }
}
impl<K: Ord + Debug, V: Debug> Validate for Node<K, V> {
    fn heap(&self) {
        self.child.iter().for_each(|child| {
            if let Some(child) = child.0.as_ref() {
                assert!(
                    self.pri <= child.pri,
                    "Broken heap condition: self = {:?}, child = {:?}",
                    self,
                    child
                );
            }
            child.heap()
        });
    }
}
