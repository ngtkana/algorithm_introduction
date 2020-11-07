mod color;
mod paren;
pub mod validate;

use color::Color;
use dbg::{lg, msg};
use std::{cmp::Ordering, fmt::Debug, rc::Rc};

pub struct PersistentRBTree<K, V>(Vec<RcNode<K, V>>);
impl<K: Ord + Debug, V: Debug> PersistentRBTree<K, V> {
    pub fn new() -> Self {
        Self(vec![RcNode(None)])
    }
    pub fn insert(&mut self, k: K, v: V) {
        let (root, e) = self.0.last().unwrap().insert(k, v, true);
        let root = root.clone_node().with_color(Color::Black).finish();
        self.0.push(root);
    }
    pub fn delete(&mut self, k: K) -> Option<Rc<(K, V)>> {
        if let Some((root, rem)) = self.0.last().unwrap().delete(k) {
            self.0.push(root);
            Some(rem)
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
        let mid = self.child(i).clone_child(1 - i);
        let orig_root = self.clone_node().with_child(i, mid).finish();
        self.child(i)
            .clone_node()
            .with_child(1 - i, orig_root)
            .finish()
    }

    // -- rb ops
    fn insert(&self, k: K, v: V, is_root: bool) -> (RcNode<K, V>, Option<DoubleRed>) {
        let res = match self.0.as_ref() {
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
        };
        res
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
                    let y = self.clone_child_node(i).with_color(Color::Black).finish();
                    let x = self
                        .clone_node()
                        .with_color(Color::Red)
                        .with_child(i, y)
                        .finish();
                    let root = x.rotate(i);
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
    fn delete(&self, k: K) -> Option<(RcNode<K, V>, Rc<(K, V)>)> {
        todo!()
    }
    fn delete_first(&self) -> Option<(RcNode<K, V>, Rc<(K, V)>)> {
        todo!()
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

#[cfg(test)]
mod tests {
    use super::{validate, PersistentRBTree};
    use rand::prelude::*;

    #[test]
    fn test_rand_small() {
        test_rand(100, 40, 42);
        test_rand(100, 40, 43);
    }

    #[test]
    fn test_rand_large() {
        test_rand(10, 200, 42);
        test_rand(10, 400, 43);
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
                match rng.gen_range(0, 2) {
                    0 => test.insert(rng.gen_range(0, 10)),
                    1 => test.insert(rng.gen_range(0, 10)),
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
