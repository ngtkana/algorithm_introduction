mod color;
mod paren;
pub mod validate;

use color::Color;
use std::{cmp::Ordering, fmt::Debug};

pub struct RBTree<K, V>(BoxedNode<K, V>);
impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    pub fn new() -> Self {
        Self(BoxedNode(None))
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v);
        self.0.set_color(Color::Black);
    }
    pub fn delete(&mut self, k: K) {
        self.0.delete(k);
        if let Some(me) = self.0.0.as_mut() {
            me.color = Color::Black;
        }
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

struct BoxedNode<K, V>(Option<Box<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> BoxedNode<K, V> {
    fn new(k: K, v: V) -> Self {
        Self(Some(Box::new(Node {
            child: [Self(None), Self(None)],
            key: k,
            value: v,
            color: Color::Red,
        })))
    }
    fn is_nil(&self) -> bool {
        self.0.is_none()
    }

    fn replace(&mut self, x: Self) -> Self {
        std::mem::replace(self, x)
    }
    fn take(&mut self) -> Self {
        self.replace(Self(None))
    }

    // unwrap
    fn unwrap(&self) -> &Node<K, V> {
        self.0.as_ref().unwrap()
    }
    fn unwrap_mut(&mut self) -> &mut Node<K, V> {
        self.0.as_mut().unwrap()
    }
    fn child(&self, i: usize) -> &Self {
        &self.unwrap().child[i]
    }
    fn child_mut(&mut self, i: usize) -> &mut Self {
        &mut self.unwrap_mut().child[i]
    }
    fn assert_isolated(self) -> Self {
        assert!(self.child(0).is_nil());
        assert!(self.child(1).is_nil());
        self
    }
    fn replace_empty_child(&mut self, i: usize, x: Self) {
        assert!(self.child(i).is_nil());
        let old = self.child_mut(i).replace(x);
        assert!(old.is_nil());
    }
    fn take_child(&mut self, i: usize) -> Self {
        self.child_mut(i).replace(Self(None))
    }
    fn set_color(&mut self, color: Color) {
        self.unwrap_mut().color = color;
    }
    fn swap_color(&mut self, y: &mut Self) {
        let x = self.unwrap_mut();
        let y = y.unwrap_mut();
        std::mem::swap(&mut x.color, &mut y.color);
    }
    fn swap_color_with_child(&mut self, i: usize) {
        let color = self.color();
        let color = std::mem::replace(&mut self.child_mut(i).unwrap_mut().color, color);
        self.unwrap_mut().color = color;
    }
    fn rotate(&mut self, i: usize) {
        let mut x = self.take();
        let mut y = x.take_child(i);
        let z = y.take_child(1 - i);
        x.replace_empty_child(i, z);
        y.replace_empty_child(1 - i, x);
        *self = y
    }
    fn swap_color_rotate(&mut self, i: usize) {
        self.swap_color_with_child(i);
        self.rotate(i);
    }

    // color
    fn color(&self) -> Color {
        self.0.as_ref().map_or(Color::Black, |x| x.color)
    }
    fn is_red(&self) -> bool {
        self.color() == Color::Red
    }
    fn is_black(&self) -> bool {
        self.color() == Color::Black
    }
    fn assert_red(&self) -> &Self {
        assert!(self.is_red());
        self
    }
    fn assert_black(&self) -> &Self {
        assert!(self.is_black());
        self
    }

    // rb ops
    fn insert(&mut self, k: K, v: V) -> Option<DoubleRed> {
        if let Some(me) = self.0.as_mut() {
            let i = if k <= me.key { 0 } else { 1 };
            let e = me.child[i].insert(k, v);
            self.insert_and_then(i, e)
        } else {
            *self = Self::new(k, v);
            Some(DoubleRed::Me)
        }
    }
    fn insert_and_then(&mut self, i: usize, e: Option<DoubleRed>) -> Option<DoubleRed> {
        match e? {
            DoubleRed::Me => match self.color() {
                Color::Red => Some(DoubleRed::Child(i)),
                Color::Black => None,
            },
            DoubleRed::Child(j) => self.insert_fixup(i, j),
        }
    }
    fn insert_fixup(&mut self, i: usize, j: usize) -> Option<DoubleRed> {
        self.assert_black()
            .child(i)
            .assert_red()
            .child(j)
            .assert_red();
        match self.child(1 - i).color() {
            Color::Red => {
                (0..2).for_each(|i| self.child_mut(i).set_color(Color::Black));
                self.set_color(Color::Red);
                Some(DoubleRed::Me)
            }
            Color::Black => {
                if i == j {
                    self.swap_color_rotate(i);
                    None
                } else {
                    self.child_mut(i).swap_color_rotate(j);
                    self.insert_fixup(i, 1 - j)
                }
            }
        }
    }
    fn delete(&mut self, k: K) -> Option<(BoxedNode<K, V>, Option<Charge>)> {
        let me = self.0.as_mut()?;
        let i = match k.cmp(&me.key) {
            Ordering::Equal => {
                return Some(if let Some((mut rem, e)) = me.child[1].delete_first() {
                    (0..2).for_each(|i| rem.replace_empty_child(i, self.take_child(i)));
                    self.swap_color(&mut rem);
                    let e = rem.delete_and_then(1, e);
                    (self.replace(rem), e)
                } else {
                    assert!(self.child(1).is_nil());
                    let child = self.take_child(0);
                    let rem = self.replace(child);
                    let e = match rem.color() {
                        Color::Red => None,
                        Color::Black => Some(Charge()),
                    };
                    (rem, e)
                });
            }
            Ordering::Less => 0,
            Ordering::Greater => 1,
        };
        let (rem, e) = me.child[i].delete(k)?;
        let e = self.delete_and_then(i, e);
        Some((rem, e))
    }
    fn delete_first(&mut self) -> Option<(BoxedNode<K, V>, Option<Charge>)> {
        let me = self.0.as_mut()?;
        Some(if let Some((rem, e)) = me.child[0].delete_first() {
            let e = self.delete_and_then(0, e);
            (rem, e)
        } else {
            assert!(self.child(0).is_nil());
            let child = self.take_child(1);
            let rem = self.replace(child).assert_isolated();
            let charge = match rem.color() {
                Color::Red => None,
                Color::Black => Some(Charge()),
            };
            (rem, charge)
        })
    }
    fn delete_and_then(&mut self, i: usize, e: Option<Charge>) -> Option<Charge> {
        e.and_then(|Charge()| self.delete_fixup(i))
    }
    fn delete_fixup(&mut self, i: usize) -> Option<Charge> {
        match self.child(i).color() {
            Color::Red => {
                self.child_mut(i).set_color(Color::Black);
                None
            }
            Color::Black => match self.child(1 - i).color() {
                Color::Red => {
                    self.assert_black().child(1 - i).child(0).assert_black();
                    self.assert_black().child(1 - i).child(1).assert_black();
                    self.swap_color_rotate(1-i);
                    let e = self.child_mut(i).delete_fixup(i);
                    self.delete_and_then(i, e)
                }
                Color::Black => match (
                    self.child(1 - i).child(i).color(),
                    self.child(1 - i).child(1 - i).color(),
                ) {
                    (Color::Black, Color::Black) => {
                        self.child_mut(1 - i).set_color(Color::Red);
                        Some(Charge())
                    }
                    (Color::Red, Color::Black) => {
                        self.child_mut(1 - i).swap_color_rotate(i);
                        self.delete_fixup(i)
                    }
                    (_, Color::Red) => {
                        self.child_mut(1 - i)
                            .child_mut(1 - i)
                            .set_color(Color::Black);
                        self.swap_color_rotate(1 - i);
                        None
                    }
                },
            },
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
}

struct Node<K, V> {
    child: [BoxedNode<K, V>; 2],
    key: K,
    value: V,
    color: Color,
}

enum DoubleRed {
    Me,
    Child(usize),
}

struct Charge();

#[cfg(test)]
mod tests {
    use super::{validate, RBTree};
    use rand::prelude::*;

    #[test]
    fn test_rand_small() {
        test_rand(100, 20, 42);
        test_rand(100, 20, 43);
        test_rand(100, 20, 91);
    }

    #[test]
    fn test_rand_large() {
        test_rand(10, 200, 42);
        test_rand(10, 200, 43);
        test_rand(10, 200, 91);
    }

    #[test]
    fn test_hand() {
        let mut test = Test::new();
        test.insert(10);
        test.insert(15);
        test.insert(13);
        test.insert(15);
        test.insert(15);
        test.insert(18);
        test.insert(15);

        test.delete(13);
        test.delete(18);
        test.delete(15);
        test.delete(13);
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut test = Test::new();
            for _ in 0..q {
                match rng.gen_range(0, 3) {
                    0 => test.insert(rng.gen_range(0, 10)),
                    1 | 2 => test.delete(rng.gen_range(0, 10)),
                    _ => unreachable!(),
                }
            }
        }
    }

    struct Test {
        rbt: RBTree<u32, ()>,
        vec: Vec<u32>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                rbt: RBTree::new(),
                vec: Vec::new(),
            }
        }
        fn insert(&mut self, k: u32) {
            println!("Insert {:?}.", &k);
            self.rbt.insert(k, ());
            let lb = match self.vec.binary_search(&k) {
                Ok(i) => i,
                Err(i) => i,
            };
            self.vec.insert(lb, k);
            self.postprocess();
        }
        fn delete(&mut self, k: u32) {
            println!("Delete {:?}.", &k);
            self.rbt.delete(k);
            match self.vec.binary_search(&k) {
                Ok(i) => {
                    self.vec.remove(i);
                }
                Err(_) => (),
            };
            self.postprocess();
        }
        fn postprocess(&self) {
            println!("rbt = {:?}", &self.rbt);
            validate::all(&self.rbt);
            assert_eq!(
                &self
                    .rbt
                    .collect_vec()
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>(),
                &self.vec,
            );
        }
    }
}
