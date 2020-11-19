use super::{BoxedNode, Node, Treap};
use rand::prelude::*;
use std::fmt::{self, Debug};

pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}
impl<K: Ord + Debug, V: Debug, R: Rng> Paren for Treap<K, V, R> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}
impl<K: Ord + Debug, V: Debug> Paren for BoxedNode<K, V> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.as_ref().iter().map(|x| x.paren(w)).collect()
    }
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
struct ParenWrapper<'a, T>(&'a T);
impl<'a, T: Paren> Debug for ParenWrapper<'a, T> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}
impl<K: Ord + Debug, V: Debug, R: Rng> Debug for Treap<K, V, R> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("Treap").field(&ParenWrapper(self)).finish()
    }
}
impl<K: Ord + Debug, V: Debug> Debug for BoxedNode<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("BoxedNode")
            .field(
                &self
                    .0
                    .as_ref()
                    .map_or("nil".to_owned(), |x| x.pri.to_string()),
            )
            .field(&ParenWrapper(self))
            .finish()
    }
}
impl<K: Ord + Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("Node").field(&ParenWrapper(self)).finish()
    }
}
