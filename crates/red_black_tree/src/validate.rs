use super::{
    color::Color,
    node::{RcNode, WeakNode},
    RBTree,
};
use std::fmt::Debug;

pub trait Validate {
    fn reflexive_parent(&self);
    fn no_double_red(&self);
    fn consistent_black_height(&self) -> u32;
    fn all(&self) {
        self.reflexive_parent();
        self.no_double_red();
        self.consistent_black_height();
    }
}

impl<K: Ord + Debug, V: Debug> Validate for RcNode<K, V> {
    fn reflexive_parent(&self) {
        let self_ref = self.as_ref();
        if let Some(internal) = self_ref.as_internal() {
            for i in 0..2 {
                let child = RcNode::clone(&internal.child(i));
                Validate::reflexive_parent(&child);
                assert!(
                    child.as_ref().parent().is_some(),
                    "No parent: self = {:?}, child = {:?}",
                    &self,
                    &child,
                );
                assert!(
                    WeakNode::ptr_eq(&RcNode::downgrade(&self), child.as_ref().parent().unwrap()),
                    "Non reflexive parent: self = {:?}, child = {:?}",
                    &self,
                    &child,
                );
            }
        }
    }
    fn no_double_red(&self) {
        let self_ref = self.as_ref();
        if let Some(internal) = self_ref.as_internal() {
            for i in 0..2 {
                let child = RcNode::clone(&internal.child(i));
                Validate::no_double_red(&child);
                assert!(
                    self.is_black() || child.is_black(),
                    "Double red: self = {:?}, child = {:?}",
                    &self,
                    &child,
                );
            }
        }
    }
    fn consistent_black_height(&self) -> u32 {
        let self_ref = self.as_ref();
        if let Some(internal) = self_ref.as_internal() {
            let x = internal.child(0).consistent_black_height();
            let y = internal.child(1).consistent_black_height();
            assert_eq!(x, y, "Inconsistent black height: self = {:?}", &self,);
            match internal.color() {
                Color::Black => x + 1,
                Color::Red => x,
            }
        } else {
            0
        }
    }
}
impl<K: Ord + Debug, V: Debug> Validate for RBTree<K, V> {
    fn reflexive_parent(&self) {
        assert!(
            self.root.index_parent().is_none(),
            "Root has a parent: {:?}",
            &self
        );
        Validate::reflexive_parent(&self.root)
    }
    fn no_double_red(&self) {
        self.root.no_double_red()
    }
    fn consistent_black_height(&self) -> u32 {
        self.root.consistent_black_height()
    }
}
