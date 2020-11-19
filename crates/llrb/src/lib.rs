mod paren;
mod validate;

pub use validate::Validate;
use {
    std::{
        cmp::Ordering,
        fmt::Debug,
        mem::{replace, swap},
    },
    yansi::Paint,
};

pub struct LLRB<K, V>(BoxNode<K, V>);
impl<K: Ord + Debug, V: Debug> LLRB<K, V> {
    pub fn new() -> Self {
        Self(BoxNode::nil())
    }
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert(key, value);
        self.0.set_color(Color::Black);
    }
    pub fn delete(&mut self, key: &K) -> Option<(K, V)> {
        let root = &mut self.0;
        if root.is_two() {
            root.set_color(Color::Red)
        }
        let res = root
            .delete(key)
            .map(|node| node.0.unwrap())
            .map(|node| (node.key, node.value));
        if !root.is_nil() {
            root.set_color(Color::Black);
        }
        res
    }
    pub fn collect_vec(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut vec = Vec::new();
        self.0.collect_vec(&mut vec);
        vec
    }
}
struct BoxNode<K, V>(Option<Box<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> BoxNode<K, V> {
    fn nil() -> Self {
        Self(None)
    }
    fn is_nil(&self) -> bool {
        self.0.is_none()
    }
    fn insert(&mut self, key: K, value: V) {
        if self.is_nil() {
            *self = Node::new(key, value, Color::Red).boxed();
        } else {
            self.child_mut(if key <= self.unwrap().key { 0 } else { 1 })
                .insert(key, value);
            // Balance right leaning
            if self.child(0).is_black() && self.child(1).is_red() {
                self.rotate(1)
            }
            // Balance left double red
            if self.child(0).is_red() && self.child(0).child(0).is_red() {
                self.rotate(0)
            }
            // Split 4-node
            if self.child(0).is_red() && self.child(1).is_red() {
                self.split_node()
            }
        }
    }
    fn delete(&mut self, key: &K) -> Option<Self> {
        if self.is_nil() {
            None
        } else {
            let i = match key.cmp(&self.unwrap().key) {
                Ordering::Less => 0,
                Ordering::Greater => 1,
                Ordering::Equal => match self.color() {
                    Color::Black => 1,
                    Color::Red => {
                        // force the next node is not 2-node
                        if self.child(1).is_two() {
                            self.move_right();
                        }
                        if key == &self.unwrap().key {
                            let res = if let Some(mut rem) = self.child_mut(1).delete_first() {
                                // swap the contents
                                swap(&mut rem.unwrap_mut().key, &mut self.unwrap_mut().key);
                                swap(&mut rem.unwrap_mut().value, &mut self.unwrap_mut().value);
                                rem
                            } else {
                                replace(self, Self::nil())
                            };
                            self.delete_fixup();
                            return Some(res);
                        } else {
                            1
                        }
                    }
                },
            };
            match self.color() {
                Color::Black => {
                    // right lean if necessry
                    if i == 1 && self.is_three() {
                        self.rotate(0);
                    }
                }
                Color::Red => {
                    // force the next node is not 2-node
                    if self.child(i).is_two() {
                        match i {
                            0 => self.move_left(),
                            1 => self.move_right(),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            let rem = self.child_mut(i).delete(key);
            self.delete_fixup();
            rem
        }
    }
    fn delete_first(&mut self) -> Option<Self> {
        if self.is_nil() {
            None
        } else if self.child(0).is_nil() {
            assert!(self.child(1).is_nil());
            Some(replace(self, Self::nil()))
        } else {
            // Force the next node is not 2-node
            if self.child(0).is_black() && self.child(0).child(0).is_black() {
                self.move_left();
            }
            let rem = self.child_mut(0).delete_first();
            self.delete_fixup();
            rem
        }
    }
    fn delete_fixup(&mut self) {
        if !self.is_nil() {
            // Balance right leaning
            if self.child(0).is_black() && self.child(1).is_red() {
                self.rotate(1);
            }
            // Split 4-node
            if self.child(0).is_red() && self.child(1).is_red() {
                self.split_node();
            }
        }
    }

    fn collect_vec(&self, vec: &mut Vec<(K, V)>)
    where
        K: Clone,
        V: Clone,
    {
        if let Some(me) = self.0.as_ref() {
            me.child[0].collect_vec(vec);
            vec.push((me.key.clone(), me.value.clone()));
            me.child[1].collect_vec(vec);
        }
    }

    // -- color
    fn color(&self) -> Color {
        self.0.as_ref().map_or(Color::Black, |node| node.color)
    }
    fn is_red(&self) -> bool {
        self.color() == Color::Red
    }
    fn is_black(&self) -> bool {
        self.color() == Color::Black
    }

    // -- unwrap
    fn unwrap(&self) -> &Node<K, V> {
        self.0.as_ref().unwrap()
    }
    fn unwrap_mut(&mut self) -> &mut Node<K, V> {
        self.0.as_mut().unwrap()
    }
    fn set_color(&mut self, color: Color) {
        self.unwrap_mut().color = color
    }

    // -- child
    fn child(&self, i: usize) -> &Self {
        &self.unwrap().child[i]
    }
    fn child_mut(&mut self, i: usize) -> &mut Self {
        &mut self.unwrap_mut().child[i]
    }
    fn take_child(&mut self, i: usize) -> Self {
        replace(&mut self.unwrap_mut().child[i], Self::nil())
    }
    fn init_child(&mut self, i: usize, x: Self) {
        let old = replace(&mut self.unwrap_mut().child[i], x);
        assert!(old.is_nil());
    }

    // -- kind
    fn is_two(&self) -> bool {
        !self.is_nil() && self.is_black() && self.child(0).is_black() && self.child(1).is_black()
    }
    fn is_three(&self) -> bool {
        !self.is_nil() && self.is_black() && self.child(0).is_red() && self.child(1).is_black()
    }

    // -- rotate
    fn rotate(&mut self, i: usize) {
        let mut x = replace(self, Self::nil());
        let mut y = x.take_child(i);
        assert!(y.is_red());
        y.set_color(x.color());
        x.set_color(Color::Red);
        let z = y.take_child(1 - i);
        x.init_child(i, z);
        y.init_child(1 - i, x);
        *self = y;
    }
    fn move_left(&mut self) {
        assert!(self.is_red());
        assert!(self.child(0).is_two());
        self.merge_node();
        if self.child(1).child(0).is_red() {
            self.child_mut(1).rotate(0);
            self.rotate(1);
            self.split_node();
        }
    }
    fn move_right(&mut self) {
        assert!(self.is_red());
        assert!(self.child(1).is_two());
        self.merge_node();
        if self.child(0).child(0).is_red() {
            self.rotate(0);
            self.child_mut(1).rotate(1);
            self.split_node();
        }
    }

    fn merge_node(&mut self) {
        assert!(self.is_red());
        (0..2).for_each(|i| assert!(self.child(i).is_black()));
        self.set_color(Color::Black);
        (0..2).for_each(|i| self.child_mut(i).set_color(Color::Red));
    }
    fn split_node(&mut self) {
        assert!(self.is_black());
        (0..2).for_each(|i| assert!(self.child(i).is_red()));
        self.set_color(Color::Red);
        (0..2).for_each(|i| self.child_mut(i).set_color(Color::Black));
    }
}
struct Node<K, V> {
    child: [BoxNode<K, V>; 2],
    key: K,
    value: V,
    color: Color,
}
impl<K: Ord + Debug, V: Debug> Node<K, V> {
    fn new(key: K, value: V, color: Color) -> Self {
        Self {
            child: [BoxNode::nil(), BoxNode::nil()],
            key,
            value,
            color,
        }
    }
    fn boxed(self) -> BoxNode<K, V> {
        BoxNode(Some(Box::new(self)))
    }
}
#[derive(Debug, Clone, PartialEq, Copy, Eq)]
enum Color {
    Red,
    Black,
}
impl Color {
    fn paint<T>(&self, x: T) -> Paint<T> {
        match self {
            Self::Red => Paint::red(x),
            Self::Black => Paint::blue(x),
        }
        .bold()
    }
}

#[cfg(test)]
mod tests {
    use super::Validate;
    use super::LLRB;
    use rand::prelude::*;

    #[test]
    fn test_hand_insert() {
        let mut test = Test::new();
        test.insert(10);
        test.insert(11);
        test.insert(18);
        test.insert(14);
        test.insert(13);
        test.insert(18);
    }

    #[test]
    fn test_hand_delete() {
        let mut test = Test::new();
        test.insert(10);
        test.insert(11);
        test.insert(15);
        test.insert(17);
        test.delete(17);
        test.delete(11);
        test.delete(12);
    }

    #[test]
    fn test_rand_small() {
        test_rand(10, 50, 42);
        test_rand(10, 50, 91);
    }

    #[test]
    fn test_rand_large() {
        test_rand(10, 200, 42);
        test_rand(10, 200, 91);
    }

    #[test]
    fn test_oneline() {
        let mut test = Test::new();
        (0..100).for_each(|i| test.insert(i));
        (0..100).for_each(|i| test.delete(i));
    }

    #[test]
    fn test_oneline_reverse() {
        let mut test = Test::new();
        (0..100).for_each(|i| test.insert(i));
        (0..100).rev().for_each(|i| test.delete(i));
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for i in 0..t {
            let mut test = Test::new();
            for j in 0..q {
                println!("Test {}, Query {}", i, j);
                match rng.gen_range(0, 2) {
                    0 => test.insert(rng.gen_range(0, 30)),
                    1 => test.delete(rng.gen_range(0, 30)),
                    _ => panic!(),
                }
            }
        }
    }

    struct Test {
        llrb: LLRB<u32, ()>,
        vec: Vec<u32>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                llrb: LLRB::new(),
                vec: Vec::new(),
            }
        }
        fn insert(&mut self, x: u32) {
            println!("Insert {:?}", &x);
            self.llrb.insert(x, ());
            println!("llrb = {:?}", &self.llrb);
            let i = self.vec.binary_search(&x).map_or_else(|e| e, |x| x);
            self.vec.insert(i, x);
            self.postprocess();
        }
        fn delete(&mut self, x: u32) {
            println!("Delete {:?}", &x);
            self.llrb.delete(&x);
            println!("alv = {:?}", &self.llrb);
            if let Ok(i) = self.vec.binary_search(&x) {
                self.vec.remove(i);
            }
            self.postprocess();
        }
        fn postprocess(&self) {
            Validate::validate(&self.llrb);
            assert_eq!(
                &self
                    .llrb
                    .collect_vec()
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>(),
                &self.vec
            );
        }
    }
}
