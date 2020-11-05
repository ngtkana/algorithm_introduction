use super::{BoxedNode, Node, RBTree};
use std::fmt::{self, Debug};

pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}
impl<K: Ord + Debug, V: Debug> Paren for BoxedNode<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match &*self.0 {
            Node::Nil => Ok(()),
            Node::Internal(internal) => {
                write!(w, "(")?;
                internal.child[0].paren(w)?;
                write!(w, "{:?}", &internal.key)?;
                internal.child[1].paren(w)?;
                write!(w, ")")
            }
        }
    }
}
impl<K: Ord + Debug, V: Debug> Debug for BoxedNode<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "BoxedNode {{ ")?;
        self.paren(w)?;
        write!(w, " }}")
    }
}
impl<K: Ord + Debug, V: Debug> Debug for RBTree<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "RBTree {{ ")?;
        self.0.paren(w)?;
        write!(w, " }}")
    }
}
