use super::{color::Color, BoxedNode, Node, RBTree};
use std::fmt::Debug;

pub trait Validate {
    fn no_double_red(&self);
    fn consistent_black_height(&self) -> u32;
    fn root_is_black(&self) {}
}

pub fn all<T: Validate>(x: &T) {
    x.no_double_red();
    x.consistent_black_height();
    x.root_is_black();
}

impl<K: Ord + Debug, V: Debug> Validate for RBTree<K, V> {
    fn no_double_red(&self) {
        self.0.no_double_red();
    }
    fn consistent_black_height(&self) -> u32 {
        self.0.consistent_black_height()
    }
    fn root_is_black(&self) {
        assert!(self.0.is_black(), "Root is not black: {:?}", self);
    }
}
impl<K: Ord + Debug, V: Debug> Validate for BoxedNode<K, V> {
    fn no_double_red(&self) {
        self.0.as_ref().iter().for_each(|x| x.no_double_red())
    }
    fn consistent_black_height(&self) -> u32 {
        self.0.as_ref().map_or(0, |x| x.consistent_black_height())
    }
}
impl<K: Ord + Debug, V: Debug> Validate for Node<K, V> {
    fn no_double_red(&self) {
        for child in &self.child {
            assert!(
                self.color == Color::Black || child.is_black(),
                "Double red: self = {:?}, child = {:?}",
                &self,
                &child
            );
            child.no_double_red();
        }
    }
    fn consistent_black_height(&self) -> u32 {
        let x = self.child[0].consistent_black_height();
        let y = self.child[1].consistent_black_height();
        assert_eq!(x, y, "Inconsistent black height: {:?}", &self);
        let res = match self.color {
            Color::Red => x,
            Color::Black => x + 1,
        };
        res
    }
}
