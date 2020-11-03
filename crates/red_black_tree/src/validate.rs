use super::{
    node::{RcNode, WeakNode},
    RBTree,
};
use std::fmt::Debug;

pub trait Validate {
    fn reflexive_parent(&self);
    fn no_double_red(&self);
    fn all(&self) {
        self.reflexive_parent();
        self.no_double_red();
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
                    self.as_ref().is_black() || child.as_ref().is_black(),
                    "Double red: self = {:?}, child = {:?}",
                    &self,
                    &child,
                );
            }
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
}
