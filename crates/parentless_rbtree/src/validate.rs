use super::{BoxedNode, Color, RBTree};
use std::fmt::Debug;

pub trait Validate {
    fn no_double_red(&self);
    fn consistent_black_height(&self) -> u32;
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
    fn consistent_black_height(&self) -> u32 {
        if let Some(internal) = self.as_internal() {
            let [x, y] = [
                internal.child[0].consistent_black_height(),
                internal.child[1].consistent_black_height(),
            ];
            assert!(x == y, "Inconsistent black height: self = {:?}", self,);
            match internal.color {
                Color::Black => x + 1,
                Color::Red => x,
            }
        } else {
            0
        }
    }
}

impl<K: Ord + Debug, V: Debug> Validate for RBTree<K, V> {
    fn no_double_red(&self) {
        self.0.no_double_red()
    }
    fn consistent_black_height(&self) -> u32 {
        self.0.consistent_black_height()
    }
}

pub fn all<T: Validate>(x: &T) {
    x.no_double_red();
    x.consistent_black_height();
}
