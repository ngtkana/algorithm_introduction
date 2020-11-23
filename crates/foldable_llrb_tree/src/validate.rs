use {
    super::{BoxNode, Color, Node, LLRB},
    std::{fmt::Debug, ops::Add},
};

pub trait Validate {
    fn validate(&self) -> u32;
}
impl<K: Ord + Debug, V: Clone + Add<Output = V> + Debug> Validate for LLRB<K, V> {
    fn validate(&self) -> u32 {
        self.0.validate()
    }
}
impl<K: Ord + Debug, V: Clone + Add<Output = V> + Debug> Validate for BoxNode<K, V> {
    fn validate(&self) -> u32 {
        self.0.as_ref().map_or(0, |x| x.validate())
    }
}
impl<K: Ord + Debug, V: Clone + Add<Output = V> + Debug> Validate for Node<K, V> {
    fn validate(&self) -> u32 {
        self.child.iter().for_each(|child| {
            assert!(
                self.color == Color::Black || child.is_black(),
                "Double red: {:?}",
                &self
            );
        });
        assert!(
            self.color == Color::Black || self.child[0].is_red() || self.child[1].is_black(),
            "Right leaning 3-node: {:?}",
            &self
        );
        let x = self.child[0].validate();
        let y = self.child[1].validate();
        assert_eq!(x, y, "Inconsistent black height: self = {:?}", &self);
        match self.color {
            Color::Black => x + 1,
            Color::Red => x,
        }
    }
}
