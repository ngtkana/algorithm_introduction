use super::{BoxedNode, Node, RBTree};
use std::fmt::{self, Debug};

pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}

impl<K: Ord + Debug, V: Debug> Paren for Node<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "(")?;
        self.child[0].paren(w)?;
        write!(w, "{:?}", &self.key)?;
        self.child[1].paren(w)?;
        write!(w, ")")
    }
}
impl<K: Ord + Debug, V: Debug> Paren for BoxedNode<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.as_ref().iter().map(|x| x.paren(w)).collect()
    }
}

impl<K: Ord + Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Node {{")?;
        self.paren(w)?;
        write!(w, " }}")
    }
}
impl<K: Ord + Debug, V: Debug> Debug for BoxedNode<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "BoxedNode {{")?;
        self.paren(w)?;
        write!(w, " }}")
    }
}
impl<K: Ord + Debug, V: Debug> Debug for RBTree<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "RBTree {{")?;
        self.0.paren(w)?;
        write!(w, " }}")
    }
}
