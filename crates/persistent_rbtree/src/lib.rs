mod color;
mod paren;
pub mod validate;

use color::Color;
use std::{cmp::Ordering, fmt::Debug, rc::Rc};

pub struct PersistentRBTree<K, V>(Vec<RcNode<K, V>>);
impl<K: Ord + Debug, V: Debug> PersistentRBTree<K, V> {
    pub fn new() -> Self {
        Self(vec![RcNode(None)])
    }
    pub fn insert(&mut self, k: K, v: V) {
        let (root, _e) = self.0.last().unwrap().insert(k, v, true);
        let root = root.clone_node().with_color(Color::Black).finish();
        self.0.push(root);
    }
    pub fn delete(&mut self, k: K) -> Option<Rc<(K, V)>> {
        if let Some((root, rem, _e)) = self.0.last().unwrap().delete(k) {
            let root = RcNode(
                root.0
                    .as_ref()
                    .map(|root| Rc::new(Node::clone(root).with_color(Color::Black))),
            );
            self.0.push(root);
            Some(Rc::clone(&rem.unwrap().kv))
        } else {
            let root = RcNode::clone(&self.0.last().unwrap());
            self.0.push(root);
            None
        }
    }
    pub fn collect_vec(&self, i: usize) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut vec = Vec::new();
        self.0[i].collect_vec(&mut vec);
        vec
    }
}

pub struct RcNode<K, V>(Option<Rc<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> Clone for RcNode<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.as_ref().map(Rc::clone))
    }
}
impl<K: Ord + Debug, V: Debug> RcNode<K, V> {
    // -- ctors
    fn from_node(node: Node<K, V>) -> Self {
        Self(Some(Rc::new(node)))
    }
    fn new_node(k: K, v: V, color: Color) -> Self {
        Self::from_node(Node {
            child: [Self(None), Self(None)],
            kv: Rc::new((k, v)),
            color,
        })
    }

    // -- unwrap
    fn unwrap(&self) -> &Node<K, V> {
        self.0.as_ref().unwrap()
    }
    fn clone_node(&self) -> Node<K, V> {
        self.unwrap().clone()
    }
    fn child(&self, i: usize) -> &Self {
        &self.unwrap().child[i]
    }
    fn clone_child(&self, i: usize) -> Self {
        Self::clone(&self.child(i))
    }
    fn clone_child_node(&self, i: usize) -> Node<K, V> {
        self.child(i).clone_node()
    }
    fn assert_isolated(self) -> Self {
        assert!(self.unwrap().child.iter().all(|child| child.0.is_none()));
        self
    }
    fn make_isolated(&self) -> Self {
        self.clone_node()
            .with_child(0, Self(None))
            .with_child(1, Self(None))
            .finish()
    }

    // -- color
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

    // -- deformation
    fn rotate(&self, i: usize) -> Self {
        let y = self.child(i).clone_child(1 - i);
        let x = self.clone_node().with_child(i, y).finish();
        self.child(i).clone_node().with_child(1 - i, x).finish()
    }
    fn swap_color_rotate(&self, i: usize) -> Self {
        let x_color = self.color();
        let y_color = self.child(i).color();
        let z = self.child(i).clone_child(1 - i);
        let x = self
            .clone_node()
            .with_child(i, z)
            .with_color(y_color)
            .finish();
        let y = self
            .clone_child_node(i)
            .with_child(1 - i, x)
            .with_color(x_color)
            .finish();
        y
    }

    // -- rb ops
    fn insert(&self, k: K, v: V, is_root: bool) -> (RcNode<K, V>, Option<DoubleRed>) {
        match self.0.as_ref() {
            None => (
                Self::new_node(k, v, if is_root { Color::Black } else { Color::Red }),
                Some(DoubleRed::Me),
            ),
            Some(me) => {
                let i = if k <= me.kv.0 { 0 } else { 1 };
                let (root, e) = me.child[i].insert(k, v, false);
                let mut root = self.clone_node().with_child(i, root).finish();
                let e = e.and_then(|e| match e {
                    DoubleRed::Me => match self.color() {
                        Color::Red => Some(DoubleRed::Child(i)),
                        Color::Black => None,
                    },
                    DoubleRed::Child(j) => {
                        let (new_root, new_e) = root.insert_fixup(i, j, is_root);
                        root = new_root;
                        new_e
                    }
                });
                (root, e)
            }
        }
    }
    fn insert_fixup(&self, i: usize, j: usize, is_root: bool) -> (Self, Option<DoubleRed>) {
        self.assert_black()
            .child(i)
            .assert_red()
            .child(j)
            .assert_red();
        match self.child(1 - i).color() {
            Color::Red => {
                let c0 = self.child(0).clone_node().with_color(Color::Black).finish();
                let c1 = self.child(1).clone_node().with_color(Color::Black).finish();
                let x = self
                    .clone_node()
                    .with_color(Color::Red)
                    .with_child(0, c0)
                    .with_child(1, c1)
                    .finish();
                (x, Some(DoubleRed::Me))
            }
            Color::Black => {
                if i == j {
                    let root = self.swap_color_rotate(i);
                    (root, None)
                } else {
                    let y = self.child(i).rotate(j);
                    self.clone_node()
                        .with_child(i, y)
                        .finish()
                        .insert_fixup(i, 1 - j, is_root)
                }
            }
        }
    }
    fn delete(&self, k: K) -> Option<(RcNode<K, V>, RcNode<K, V>, Option<Charge>)> {
        let me = self.0.as_ref()?;
        let cmp = k.cmp(&me.kv.0);
        let i = match cmp {
            Ordering::Equal => {
                let res = if let Some((child, rem, e)) = me.child[1].delete_first() {
                    let (root, e) = rem
                        .assert_isolated()
                        .clone_node()
                        .with_child(0, self.clone_child(0))
                        .with_child(1, child)
                        .with_color(self.color())
                        .finish()
                        .delete_and_then(1, e);
                    (root, self.make_isolated(), e)
                } else {
                    let charge = match me.color {
                        Color::Red => None,
                        Color::Black => Some(Charge()),
                    };
                    (self.clone_child(0), self.make_isolated(), charge)
                };
                return Some(res);
            }
            Ordering::Less => 0,
            Ordering::Greater => 1,
        };
        me.child[i].delete(k).map(|(child, rem, e)| {
            let (root, e) = self
                .clone_node()
                .with_child(i, child)
                .finish()
                .delete_and_then(i, e);
            (root, rem, e)
        })
    }
    fn delete_first(&self) -> Option<(RcNode<K, V>, RcNode<K, V>, Option<Charge>)> {
        let me = self.0.as_ref()?;
        Some(match me.child[0].delete_first() {
            None => {
                let e = match me.color {
                    Color::Red => None,
                    Color::Black => Some(Charge()),
                };
                let root = me.child[1].clone();
                (root, self.make_isolated(), e)
            }
            Some((root, rem, e)) => {
                let (root, e) = self
                    .clone_node()
                    .with_child(0, root)
                    .finish()
                    .delete_and_then(0, e);
                (root, rem.assert_isolated(), e)
            }
        })
    }
    fn delete_and_then(self, i: usize, e: Option<Charge>) -> (Self, Option<Charge>) {
        match e {
            Some(Charge()) => self.delete_fixup(i),
            None => (self, None),
        }
    }
    fn delete_fixup(&self, i: usize) -> (Self, Option<Charge>) {
        match self.child(i).color() {
            Color::Red => {
                let y = self.clone_child_node(i).with_color(Color::Black).finish();
                (self.clone_node().with_child(i, y).finish(), None)
            }
            Color::Black => match self.child(1 - i).color() {
                Color::Red => {
                    let x = self.swap_color_rotate(1 - i);
                    let (y, e) = x.child(i).delete_fixup(i);
                    x.clone_node()
                        .with_child(i, y)
                        .finish()
                        .delete_and_then(i, e)
                }
                Color::Black => match (
                    self.child(1 - i).child(i).color(),
                    self.child(1 - i).child(1 - i).color(),
                ) {
                    (Color::Black, Color::Black) => {
                        let w = self.clone_child_node(1 - i).with_color(Color::Red).finish();
                        let x = self.clone_node().with_child(1 - i, w).finish();
                        (x, Some(Charge()))
                    }
                    (Color::Red, Color::Black) => {
                        let w = self.child(1 - i).swap_color_rotate(i);
                        let x = self.clone_node().with_child(1 - i, w).finish();
                        x.delete_fixup(i)
                    }
                    (_, Color::Red) => {
                        let w1 = self
                            .child(1 - i)
                            .clone_child_node(1 - i)
                            .with_color(Color::Black)
                            .finish();
                        let w = self
                            .child(1 - i)
                            .clone_node()
                            .with_child(1 - i, w1)
                            .finish();
                        let x = self
                            .clone_node()
                            .with_child(1 - i, w)
                            .finish()
                            .swap_color_rotate(1 - i);
                        (x, None)
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
            vec.push((*me.kv).clone());
            me.child[1].collect_vec(vec);
        }
    }
}

struct Node<K, V> {
    child: [RcNode<K, V>; 2],
    kv: Rc<(K, V)>,
    color: Color,
}
impl<K: Ord + Debug, V: Debug> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Self {
            child: [RcNode::clone(&self.child[0]), RcNode::clone(&self.child[1])],
            kv: Rc::clone(&self.kv),
            color: self.color,
        }
    }
}
impl<K: Ord + Debug, V: Debug> Node<K, V> {
    fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    fn with_child(mut self, i: usize, x: RcNode<K, V>) -> Self {
        self.child[i] = x;
        self
    }
    fn finish(self) -> RcNode<K, V> {
        RcNode(Some(Rc::new(self)))
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Eq)]
enum DoubleRed {
    Me,
    Child(usize),
}
#[derive(Debug, Clone, PartialEq, Copy, Eq)]
struct Charge();

#[cfg(test)]
mod tests {
    use super::{validate, PersistentRBTree};
    use rand::prelude::*;

    #[test]
    fn test_rand_small() {
        test_rand(400, 40, 42);
        test_rand(400, 40, 43);
    }

    #[test]
    fn test_rand_large() {
        test_rand(4, 400, 42);
        test_rand(4, 400, 43);
    }

    #[test]
    fn test_hand() {
        let mut test = Test::new();

        test.insert(10);
        test.insert(13);
        test.insert(11);
        test.insert(15);
        test.insert(13);
        test.insert(17);
        test.insert(12);

        test.delete(12);
        test.delete(11);
        test.delete(10);
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut test = Test::new();
            for _ in 0..q {
                match rng.gen_range(0, 4) {
                    0 => test.insert(rng.gen_range(0, 10)),
                    1 | 2 | 3 => test.delete(rng.gen_range(0, 10)),
                    _ => unreachable!(),
                }
            }
        }
    }

    struct Test {
        time: u32,
        rbt: PersistentRBTree<u32, ()>,
        vec: Vec<Vec<u32>>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                time: 0,
                rbt: PersistentRBTree::new(),
                vec: vec![Vec::new()],
            }
        }
        fn postprocess(&mut self) {
            self.time += 1;
            println!(
                "rbst[{}] = {:?}",
                self.time, &self.rbt.0[self.time as usize],
            );
            validate::all(&self.rbt);
            for i in 0..=self.time as usize {
                let result = self
                    .rbt
                    .collect_vec(i)
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>();
                let expected = self.vec[i].clone();
                assert_eq!(result, expected, "Time = {}/{}", i, self.time);
            }
        }
        fn insert(&mut self, k: u32) {
            println!("Insert {:?}", k);
            self.rbt.insert(k, ());
            let mut v = self.vec.last().unwrap().clone();
            let lb = v.binary_search(&k).map_or_else(|e| e, |x| x);
            v.insert(lb, k);
            self.vec.push(v);

            self.postprocess();
        }
        pub fn delete(&mut self, k: u32) {
            println!("Delete {:?}", k);
            let res = self.rbt.delete(k);
            println!("res = {:?}", res);
            let mut v = self.vec.last().unwrap().clone();
            let lb = v.binary_search(&k).map_or_else(|e| e, |x| x);
            if v.get(lb).map_or(false, |x| x == &k) {
                v.remove(lb);
            }
            self.vec.push(v);
            self.postprocess();
        }
    }
}
