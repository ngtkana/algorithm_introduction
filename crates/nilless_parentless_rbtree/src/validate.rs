use super::{BoxedNode, Color, Node, RBTree};
use std::fmt::Debug;

pub trait Validate {
    fn no_double_red(&self);
    fn consistent_black_height(&self) -> u32;
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
    fn consistent_black_height(&self) -> u32 {
        let x = self.child[0].consistent_black_height();
        let y = self.child[1].consistent_black_height();
        assert_eq!(x, y, "Inconsistent black height: self = {:?}", self);
        match self.color {
            Color::Red => x,
            Color::Black => x + 1,
        }
    }
}

impl<K: Ord + Debug, V: Debug> Validate for BoxedNode<K, V> {
    fn no_double_red(&self) {
        if let Some(me) = self.0.as_ref() {
            me.no_double_red()
        }
    }
    fn consistent_black_height(&self) -> u32 {
        if let Some(me) = self.0.as_ref() {
            me.consistent_black_height()
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

pub fn all<T: Debug + Validate>(x: &T) {
    println!("Validating {:?}", x);
    x.no_double_red();
    x.consistent_black_height();
}
