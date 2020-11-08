use super::{Node, PersistentRBTree, RcNode};
use std::fmt::{self, Debug};

pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}
impl<K: Ord + Debug, V: Debug> Paren for Node<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "(")?;
        self.child[0].paren(w)?;
        write!(w, "{:?}", self.color.paint(&self.kv.0))?;
        self.child[1].paren(w)?;
        write!(w, ")")
    }
}
impl<K: Ord + Debug, V: Debug> Paren for RcNode<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .as_ref()
            .iter()
            .map(|internal| internal.paren(w))
            .collect()
    }
}
struct ParenWrapper<'a, T>(&'a T);
impl<'a, T: Paren> Debug for ParenWrapper<'a, T> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}

impl<K: Ord + Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("Node").field(&ParenWrapper(self)).finish()
    }
}
impl<K: Ord + Debug, V: Debug> Debug for RcNode<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("RcNode").field(&ParenWrapper(self)).finish()
    }
}
impl<K: Ord + Debug, V: Debug> Debug for PersistentRBTree<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_list()
            .entries(self.0.iter().map(|x| ParenWrapper(x)))
            .finish()
    }
}
