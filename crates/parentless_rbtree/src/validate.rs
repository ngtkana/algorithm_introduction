use super::{BoxedNode, RBTree};
use std::fmt::Debug;

pub trait Validate {
    fn no_double_red(&self);
}

impl<K: Ord + Debug, V: Debug> Validate for BoxedNode<K, V> {
    fn no_double_red(&self) {
        if let Some(internal) = self.as_internal() {
            for i in 0..2 {
                let child = &internal.child[i];
                assert!(
                    self.is_black() || child.is_black(),
                    "Double red: self = {:?}, child = {:?}",
                    self,
                    child,
                );
                child.no_double_red()
            }
        }
    }
}

impl<K: Ord + Debug, V: Debug> Validate for RBTree<K, V> {
    fn no_double_red(&self) {
        self.0.no_double_red()
    }
}

pub fn all<T: Validate>(x: &T) {
    x.no_double_red()
}
