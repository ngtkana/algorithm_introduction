use std::fmt::{Debug, Formatter, Result};

pub struct Wrapper<'a, T>(pub &'a T);

pub trait Paren: Sized {
    fn paren(&self, w: &mut Formatter) -> Result;
    fn to_paren(&self) -> String {
        format!("{:?}", &Wrapper(self))
    }
}
impl<'a, T: Paren> Debug for Wrapper<'a, T> {
    fn fmt(&self, w: &mut Formatter) -> Result {
        self.0.paren(w)
    }
}
