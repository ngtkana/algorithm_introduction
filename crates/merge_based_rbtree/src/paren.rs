use super::{color::Color, BoxNode, Node, RBTree};
use std::fmt::{self, Debug};
use yansi::Paint;

impl<K: Ord + Debug> Debug for RBTree<K> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("RBTree").field(&ParenWrapper(self)).finish()
    }
}
impl<K: Ord + Debug> Debug for BoxNode<K> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("BoxNode").field(&ParenWrapper(self)).finish()
    }
}
impl<K: Ord + Debug> Debug for Node<K> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        w.debug_tuple("Node").field(&ParenWrapper(self)).finish()
    }
}
struct ParenWrapper<'a, T>(&'a T);
impl<'a, T: Paren> Debug for ParenWrapper<'a, T> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}
pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}
impl<K: Ord + Debug> Paren for RBTree<K> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.as_ref().iter().map(|x| x.paren(w)).collect()
    }
}
impl<K: Ord + Debug> Paren for BoxNode<K> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Internal(internal) => internal.paren(w),
            Self::Leaf(leaf) => write!(
                w,
                "{}{:?}{}",
                Color::Black.paint("("),
                Paint::yellow(&leaf.key),
                Color::Black.paint(")")
            ),
        }
    }
}
impl<K: Ord + Debug> Paren for Node<K> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{}", self.color.paint("("))?;
        self.child[0].paren(w)?;
        write!(w, "{}", self.color.paint(self.bh))?;
        self.child[1].paren(w)?;
        write!(w, "{}", self.color.paint(")"))
    }
}
