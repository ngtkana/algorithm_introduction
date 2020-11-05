use super::{BoxedNode, Color, Node, RBTree};
use std::fmt::Debug;

pub trait Validate {
    fn no_double_red(&self);
}

impl<K: Ord + Debug, V: Debug> Validate for Node<K, V> {
    fn no_double_red(&self) {
        self.child.iter().for_each(|child| {
            assert!(
                self.color == Color::Black || child.is_black(),
                "Double red: self = {:?}, child = {:?}",
                self,
                child
            );
            child.no_double_red()
        })
    }
}

impl<K: Ord + Debug, V: Debug> Validate for BoxedNode<K, V> {
    fn no_double_red(&self) {
        if let Some(me) = self.0.as_ref() {
            me.no_double_red()
        }
    }
}

impl<K: Ord + Debug, V: Debug> Validate for RBTree<K, V> {
    fn no_double_red(&self) {
        self.0.no_double_red()
    }
}

pub fn all<T: Debug + Validate>(x: &T) {
    println!("Validating {:?}", x);
    x.no_double_red();
}
