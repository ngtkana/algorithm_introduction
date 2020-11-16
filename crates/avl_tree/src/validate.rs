use super::{AVLTree, BoxedNode, Node};
use std::fmt::Debug;

pub fn all<T: Validate>(x: &T) {
    x.balanced()
}

pub trait Validate {
    fn balanced(&self);
}
impl<K: Ord + Debug, V: Debug> Validate for AVLTree<K, V> {
    fn balanced(&self) {
        self.0.balanced()
    }
}
impl<K: Ord + Debug, V: Debug> Validate for BoxedNode<K, V> {
    fn balanced(&self) {
        self.0.as_ref().iter().for_each(|x| x.balanced())
    }
}
impl<K: Ord + Debug, V: Debug> Validate for Node<K, V> {
    fn balanced(&self) {
        assert!(
            (self.child[0].ht() as i32 - self.child[1].ht() as i32).abs() <= 1,
            "Unbalanced: self = {:?}",
            self
        );
        self.child.iter().for_each(|x| x.balanced());
    }
}
