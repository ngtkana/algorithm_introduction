use super::{
    node::{RcNode, WeakNode},
    RBTree,
};
use std::fmt::Debug;

pub trait Validate {
    fn reflexive_parent(&self);
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
                    &internal.key(),
                    &i,
                );
            }
        }
    }
}
impl<K: Ord + Debug, V: Debug> Validate for RBTree<K, V> {
    fn reflexive_parent(&self) {
        Validate::reflexive_parent(&self.root)
    }
}
