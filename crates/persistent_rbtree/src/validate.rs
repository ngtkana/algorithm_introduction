use super::{Color, Node, PersistentRBTree, RcNode};
use std::fmt::Debug;

pub trait Validate {
    fn root_is_black(&self) {}
    fn no_double_red(&self);
    fn consistent_black_height(&self) -> u32;
}

pub fn all<T: Validate>(x: &T) {
    x.root_is_black();
    x.no_double_red();
    x.consistent_black_height();
}

impl<K: Ord + Debug, V: Debug> Validate for Node<K, V> {
    fn no_double_red(&self) {
        for child in self.child.iter() {
            child.no_double_red();
            assert!(
                self.color == Color::Black || child.is_black(),
                "Double red: self = {:?}, child = {:?}",
                self,
                child
            );
        }
    }
    fn consistent_black_height(&self) -> u32 {
        let x = self.child[0].consistent_black_height();
        let y = self.child[1].consistent_black_height();
        assert_eq!(x, y, "self = {:?}", &self);
        x
    }
}
impl<K: Ord + Debug, V: Debug> Validate for RcNode<K, V> {
    fn no_double_red(&self) {
        self.0.as_ref().iter().for_each(|x| x.no_double_red())
    }
    fn consistent_black_height(&self) -> u32 {
        self.0.as_ref().map_or(0, |x| x.consistent_black_height())
    }
}
impl<K: Ord + Debug, V: Debug> Validate for PersistentRBTree<K, V> {
    fn no_double_red(&self) {
        self.0.iter().for_each(Validate::no_double_red)
    }
    fn consistent_black_height(&self) -> u32 {
        self.0
            .iter()
            .map(Validate::consistent_black_height)
            .for_each(|_| {});
        0
    }
    fn root_is_black(&self) {
        assert!(
            self.0.iter().all(|root| root.is_black()),
            "Root is not black: {:?}",
            &self
        )
    }
}
