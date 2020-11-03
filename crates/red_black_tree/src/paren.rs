use super::{
    node::{Node, RcNode, WeakNode},
    RBTree,
};
use std::fmt::{self, Debug};

pub trait ParenInternal {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}

impl<K: Ord + Debug, V: Debug> ParenInternal for RcNode<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match &*self.as_ref() {
            Node::Nil(_) => (),
            Node::Internal(internal) => {
                write!(w, "(")?;
                internal.child(0).paren(w)?;
                write!(w, "{:?}", internal.color().paint(internal.key()))?;
                internal.child(1).paren(w)?;
                write!(w, ")")?;
            }
        }
        Ok(())
    }
}

impl<K: Ord + Debug, V: Debug> Debug for RBTree<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "RBTree {{ ")?;
        ParenInternal::paren(&self.root, w)?;
        write!(w, " }}")
    }
}

impl<K: Ord + Debug, V: Debug> Debug for RcNode<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "RcNode {{ tree: ")?;
        ParenInternal::paren(self, w)?;
        write!(w, ", parent:")?;
        if let Some(p) = self
            .as_ref()
            .parent()
            .map(|p| WeakNode::upgrade(&p).unwrap())
        {
            write!(w, "Some({:?})", p.as_ref().as_internal().unwrap().key())?;
        } else {
            write!(w, "None")?;
        }
        write!(w, " }}")
    }
}
