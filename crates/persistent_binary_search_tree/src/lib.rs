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
        let root = self.0.last().unwrap().insert(k, v);
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
    fn from_kv(k: K, v: V) -> Self {
        Self::from_node(Node {
            child: [Self(None), Self(None)],
            kv: Rc::new((k, v)),
            color: Color::Red,
        })
    }
    fn from_links(kv: Rc<(K, V)>, i: usize, l: RcNode<K, V>, r: RcNode<K, V>) -> Self {
        Self::from_node(Node {
            child: match i {
                0 => [l, r],
                1 => [r, l],
                _ => panic!(),
            },
            kv,
            color: Color::Red,
        })
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

    // -- rb ops
    fn insert(&self, k: K, v: V) -> RcNode<K, V> {
        match self.0.as_ref() {
            None => Self::from_kv(k, v),
            Some(me) => {
                let i = if k <= me.kv.0 { 0 } else { 1 };
                Self::from_links(
                    Rc::clone(&me.kv),
                    i,
                    me.child[i].insert(k, v),
                    RcNode::clone(&me.child[1 - i]),
                )
            }
        }
    }
    fn delete(&self, k: K) -> Option<(RcNode<K, V>, Rc<(K, V)>)> {
        let me = self.0.as_ref()?;
        let cmp = k.cmp(&me.kv.0);
        let i = match cmp {
            Ordering::Equal => {
                let child_res = me.child[1].delete_first();
                let root = match child_res {
                    Some((root, kv)) => Self::from_links(kv, 0, RcNode::clone(&me.child[0]), root),
                    None => RcNode::clone(&me.child[0]),
                };
                return Some((root, Rc::clone(&me.kv)));
            }
            Ordering::Less => 0,
            Ordering::Greater => 1,
        };
        me.child[i].delete(k).map(|(root, rem)| {
            (
                Self::from_links(Rc::clone(&me.kv), i, root, RcNode::clone(&me.child[1 - i])),
                rem,
            )
        })
    }
    fn delete_first(&self) -> Option<(RcNode<K, V>, Rc<(K, V)>)> {
        let me = self.0.as_ref()?;
        let child_res = me.child[0].delete_first();
        let res = child_res
            .map(|(root, kv)| {
                (
                    Self::from_links(Rc::clone(&me.kv), 0, root, RcNode::clone(&me.child[1])),
                    kv,
                )
            })
            .unwrap_or_else(|| (RcNode::clone(&me.child[1]), Rc::clone(&me.kv)));
        Some(res)
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
