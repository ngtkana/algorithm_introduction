use super::{color::Color, BoxNode, Node, RBTree};
use std::fmt::Debug;

pub fn all<T: Validate>(x: &T) {
    x.no_double_red();
    x.correct_black_height();
    x.correct_size();
    x.root_is_black();
}

impl<K: Ord + Debug> Validate for RBTree<K> {
    fn no_double_red(&self) {
        self.0.as_ref().iter().for_each(|root| root.no_double_red());
    }
    fn correct_black_height(&self) {
        self.0
            .as_ref()
            .iter()
            .for_each(|root| root.correct_black_height());
    }
    fn correct_size(&self) {
        self.0.as_ref().iter().for_each(|root| root.correct_size());
    }
    fn root_is_black(&self) {
        assert!(self.0.as_ref().map_or(true, |root| root.is_black()));
    }
}

impl<K: Ord + Debug> Validate for BoxNode<K> {
    fn no_double_red(&self) {
        self.as_node().iter().for_each(|node| node.no_double_red());
    }
    fn correct_black_height(&self) {
        self.as_node()
            .iter()
            .for_each(|node| node.correct_black_height());
    }
    fn correct_size(&self) {
        self.as_node().iter().for_each(|node| node.correct_size());
    }
}

impl<K: Ord + Debug> Validate for Node<K> {
    fn no_double_red(&self) {
        self.child.iter().for_each(|child| {
            assert!(
                child.is_black() || self.color == Color::Black,
                "Double red: self = {:?}, child ={:?}",
                &self,
                child
            );
            child.no_double_red();
        });
    }
    fn correct_black_height(&self) {
        self.child.iter().for_each(|child| {
            assert_eq!(
                child.bh_aug(),
                self.bh,
                "Incorrect black height: self = {:?}, child ={:?}",
                &self,
                child
            );
            child.correct_black_height();
        });
    }
    fn correct_size(&self) {
        assert_eq!(
            self.child.iter().map(|child| child.size()).sum::<usize>(),
            self.size,
            "Incorrect size: self = {:?}",
            &self,
        );
        self.child.iter().for_each(|child| {
            child.correct_size();
        });
    }
}

pub trait Validate {
    fn no_double_red(&self);
    fn correct_black_height(&self);
    fn correct_size(&self);
    fn root_is_black(&self) {}
}
