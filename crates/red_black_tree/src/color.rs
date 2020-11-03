use yansi::Paint;

#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub enum Color {
    Red,
    Black,
}
impl Color {
    pub fn paint<T>(&self, x: T) -> Paint<T> {
        match self {
            Color::Red => Paint::red(x),
            Color::Black => Paint::blue(x),
        }
        .bold()
    }
}
