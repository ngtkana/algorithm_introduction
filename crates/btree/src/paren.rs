use {
    super::{BTree, Node},
    std::fmt::{self, Debug},
};

// impl<K: Ord + Debug> Debug for BTree<K> {
//     fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
//         w.debug_tuple("BTree").field(&Wrapper(self)).finish()
//     }
// }
// impl<K: Ord + Debug> Debug for Node<K> {
//     fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
//         w.debug_tuple("Node").field(&Wrapper(self)).finish()
//     }
// }
pub struct Wrapper<'a, T>(pub &'a T);
impl<'a, T: Paren> Debug for Wrapper<'a, T> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}

pub trait Paren {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result;
}
impl<K: Ord + Debug> Paren for BTree<K> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.paren(w)
    }
}
impl<K: Ord + Debug> Paren for Node<K> {
    fn paren(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "[")?;
        if self.is_leaf() {
            for i in 0..self.keys.len() {
                if i != 0 {
                    write!(w, ",")?;
                }
                write!(w, "{:?}", &self.keys[i])?;
            }
        } else {
            for i in 0..self.keys.len() {
                self.child[i].paren(w)?;
                write!(w, "{:?}", &self.keys[i])?;
            }
            self.child.back().unwrap().paren(w)?;
        }
        write!(w, "]")
    }
}
