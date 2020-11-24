mod paren;

use std::fmt::Debug;

const ORDER: usize = 2;
const MIN_KV: usize = ORDER - 1;
const MIN_CHILD: usize = ORDER;
const MAX_KV: usize = ORDER * 2 - 1;
const MAX_CHILD: usize = ORDER * 2;

pub struct BTree<K>(Node<K>);
impl<K: Ord + Debug> BTree<K> {
    pub fn new() -> Self {
        Self(Node {
            keys: Vec::new(),
            child: None,
        })
    }
    pub fn insert(&mut self, key: K) {
        if self.0.is_saturated() {
            let mut left = std::mem::replace(&mut self.0, Node::new());
            let (mid, right) = left.split_off();
            let root = Node {
                keys: vec![mid],
                child: Some(vec![Box::new(left), Box::new(right)]),
            };
            self.0 = root;
        }
        self.0.insert(key)
    }
}

struct Node<K> {
    keys: Vec<K>,
    child: Option<Vec<Box<Node<K>>>>,
}
impl<K: Ord + Debug> Node<K> {
    fn new() -> Self {
        Node {
            keys: Vec::new(),
            child: None,
        }
    }
    fn is_leaf(&self) -> bool {
        self.child.is_none()
    }
    fn is_saturated(&self) -> bool {
        self.keys.len() == MAX_KV
    }
    fn insert(&mut self, key: K) {
        dbg::msg!("insert", (&self, &key));
        let mut i = self
            .keys
            .iter()
            .position(|k| &key <= k)
            .unwrap_or(self.keys.len());
        if self.is_leaf() {
            self.keys.insert(i, key);
        } else {
            if self.child()[i].is_saturated() {
                self.split_child(i);
                if self.keys[i] < key {
                    i += 1;
                }
            }
            self.child_mut()[i].insert(key);
        }
    }
    fn split_off(&mut self) -> (K, Self) {
        dbg::msg!("split_off", &self);
        let child = self.child.as_mut().map(|child| child.split_off(MIN_CHILD));
        let keys = self.keys.split_off(MIN_KV + 1);
        let mid = self.keys.pop().unwrap();
        (mid, Self { keys, child: child })
    }
    fn split_child(&mut self, i: usize) {
        let (mid, node) = self.child_mut()[i].split_off();
        self.child_mut().insert(i + 1, Box::new(node));
        self.keys.insert(i, mid);
    }
    fn child(&self) -> &Vec<Box<Self>> {
        self.child.as_ref().unwrap()
    }
    fn child_mut(&mut self) -> &mut Vec<Box<Self>> {
        self.child.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use {super::BTree, rand::prelude::*};

    #[test]
    fn test_rand_small() {
        test_hand(1, 40, 42);
    }

    fn test_hand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut test = Test::new();
            for _ in 0..q {
                test.insert(rng.gen_range(0, 30));
            }
        }
    }

    struct Test {
        bt: BTree<u32>,
        vec: Vec<u32>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                bt: BTree::new(),
                vec: Vec::new(),
            }
        }
        fn insert(&mut self, x: u32) {
            println!("Insert {:?}", &x);
            self.bt.insert(x);
            println!("bt = {:?}", &self.bt);
            let i = self.vec.binary_search(&x).map_or_else(|e| e, |x| x);
            self.vec.insert(i, x);
            self.postprocess();
        }
        fn postprocess(&self) {
            // assert_eq!(
            //     &self
            //         .bt
            //         .collect_vec()
            //         .iter()
            //         .map(|&(k, ())| k)
            //         .collect::<Vec<_>>(),
            //     &self.vec
            // );
        }
    }
}
