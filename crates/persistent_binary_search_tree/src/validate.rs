use super::{Color, Node, PersistentRBTree, RcNode};
use std::fmt::Debug;

pub trait Validate {
    fn no_double_red(&self);
    fn root_is_black(&self) {}
}

pub fn all<T: Validate>(x: &T) {
    x.root_is_black();
    x.no_double_red();
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
}
impl<K: Ord + Debug, V: Debug> Validate for RcNode<K, V> {
    fn no_double_red(&self) {
        self.0.as_ref().iter().for_each(|x| x.no_double_red())
    }
}
impl<K: Ord + Debug, V: Debug> Validate for PersistentRBTree<K, V> {
    fn no_double_red(&self) {
        self.0.iter().for_each(Validate::no_double_red)
    }
    fn root_is_black(&self) {
        assert!(
            self.0.iter().all(|root| root.is_black()),
            "Root is not black: {:?}",
            &self
        )
    }
}
