use {
    super::{BTree, Node},
    std::fmt::{self, Debug},
};

pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}
impl<K: Ord + Debug> Paren for BTree {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}
impl<K: Ord + Debug> Paren for BTree {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if let Some(child) = self.child.as_ref() {
            child.iter().for_each(|
    }
}
