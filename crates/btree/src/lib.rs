pub mod paren;

use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::Debug,
    mem::{replace, swap},
};

const ORDER: usize = 4;
const MIN_KEYS: usize = ORDER - 1;
const MAX_KEYS: usize = ORDER * 2 - 1;

#[derive(Debug)]
pub struct BTree<K>(Node<K>);
impl<K: Ord + Debug> BTree<K> {
    pub fn new() -> Self {
        Self(Node {
            keys: VecDeque::new(),
            child: VecDeque::new(),
        })
    }
    pub fn insert(&mut self, key: K) -> Option<&K> {
        if self.0.is_saturated() {
            let mut left = replace(&mut self.0, Node::new());
            let (mid, right) = left.split_off();
            let root = Node {
                keys: VecDeque::from(vec![mid]),
                child: VecDeque::from(vec![Box::new(left), Box::new(right)]),
            };
            self.0 = root;
        }
        self.0.insert(key)
    }
    pub fn delete(&mut self, key: K) -> Option<K> {
        let res = self.0.delete(key);
        if self.0.keys.is_empty() {
            if let Some(child) = self.0.child.pop_back() {
                assert!(self.0.child.is_empty());
                self.0 = *child;
            }
        }
        res
    }
    pub fn collect_vec(&self) -> Vec<K>
    where
        K: Clone,
    {
        let mut vec = Vec::new();
        self.0.collect_vec(&mut vec);
        vec
    }
}
impl<K: Ord + Debug> Default for BTree<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Node<K> {
    keys: VecDeque<K>,
    child: VecDeque<Box<Node<K>>>,
}
impl<K: Ord + Debug> Node<K> {
    fn new() -> Self {
        Node {
            keys: VecDeque::new(),
            child: VecDeque::new(),
        }
    }
    fn is_leaf(&self) -> bool {
        self.child.is_empty()
    }
    fn is_narrow(&self) -> bool {
        self.keys.len() == MIN_KEYS
    }
    fn is_saturated(&self) -> bool {
        self.keys.len() == MAX_KEYS
    }
    fn insert(&mut self, key: K) -> Option<&K> {
        match linear_search(&self.keys, &key) {
            Ok(_) => None,
            Err(pos) => {
                if self.is_leaf() {
                    self.keys.insert(pos, key);
                    Some(&self.keys[pos])
                } else {
                    let mut pos = pos;
                    if self.child[pos].is_saturated() {
                        self.split_child(pos);
                        match key.cmp(&self.keys[pos]) {
                            Ordering::Less => (),
                            Ordering::Equal => return None,
                            Ordering::Greater => pos += 1,
                        }
                    }
                    self.child[pos].insert(key)
                }
            }
        }
    }
    fn delete(&mut self, key: K) -> Option<K> {
        if self.is_leaf() {
            let pos = self.keys.iter().position(|x| *x == key)?;
            Some(self.keys.remove(pos).unwrap())
        } else {
            match linear_search(&self.keys, &key) {
                Ok(pos) => match self.widen_child(pos + 1).checked_sub(1) {
                    Some(pos) => {
                        if self.keys.get(pos).map_or(false, |x| *x == key) {
                            let rem = self.child[pos + 1].delete_first();
                            Some(replace(&mut self.keys[pos], rem))
                        } else {
                            self.child[pos + 1].delete(key)
                        }
                    }
                    None => self.child[0].delete(key),
                },
                Err(pos) => {
                    let pos = self.widen_child(pos);
                    if self.keys.get(pos).map_or(false, |x| *x == key) {
                        let rem = self.child[pos + 1].delete_first();
                        Some(replace(&mut self.keys[pos], rem))
                    } else {
                        self.child[pos].delete(key)
                    }
                }
            }
        }
    }
    fn delete_first(&mut self) -> K {
        if self.is_leaf() {
            self.keys.pop_front().unwrap()
        } else {
            self.widen_child(0);
            self.child[0].delete_first()
        }
    }
    fn collect_vec(&self, vec: &mut Vec<K>)
    where
        K: Clone,
    {
        if self.is_leaf() {
            vec.extend(self.keys.iter().cloned())
        } else {
            for i in 0..self.keys.len() {
                self.child[i].collect_vec(vec);
                vec.push(self.keys[i].clone());
            }
            self.child.back().unwrap().collect_vec(vec);
        }
    }
    // widen the i-th child and return the new index (different from the original one when merged with the previous one)
    fn widen_child(&mut self, mut i: usize) -> usize {
        if self.child[i].is_narrow() {
            if i + 1 < self.child.len() {
                if self.child[i + 1].keys.len() == MIN_KEYS {
                    self.merge_child(i);
                } else {
                    self.move_from_right(i);
                }
            } else if self.child[i - 1].keys.len() == MIN_KEYS {
                self.merge_child(i - 1);
                i -= 1;
            } else {
                self.move_from_left(i);
            }
        }
        i
    }
    // move branch (i + 1)-th to i-th
    fn move_from_right(&mut self, i: usize) {
        let (mut key, child) = self.child[i + 1].pop_front();
        swap(&mut self.keys[i], &mut key);
        self.child[i].push_back(key, child);
    }
    // move branch (i + 1)-th to i-th
    fn move_from_left(&mut self, i: usize) {
        let (mut key, child) = self.child[i - 1].pop_back();
        swap(&mut self.keys[i - 1], &mut key);
        self.child[i].push_front(key, child);
    }
    fn pop_front(&mut self) -> (K, Option<Box<Self>>) {
        let child = self.child.pop_front();
        let key = self.keys.pop_front().unwrap();
        (key, child)
    }
    fn pop_back(&mut self) -> (K, Option<Box<Self>>) {
        let child = self.child.pop_back();
        let key = self.keys.pop_back().unwrap();
        (key, child)
    }
    fn push_front(&mut self, key: K, child: Option<Box<Self>>) {
        if let Some(child) = child {
            self.child.push_front(child);
        }
        self.keys.push_front(key);
    }
    fn push_back(&mut self, key: K, child: Option<Box<Self>>) {
        if let Some(child) = child {
            self.child.push_back(child);
        }
        self.keys.push_back(key);
    }
    // i and i + 1
    fn merge_child(&mut self, i: usize) {
        let key = self.keys.remove(i).unwrap();
        let child = self.child.remove(i + 1).unwrap();
        self.child[i].append(key, *child);
    }
    fn split_child(&mut self, i: usize) {
        let (mid, node) = self.child[i].split_off();
        self.child.insert(i + 1, Box::new(node));
        self.keys.insert(i, mid);
    }
    fn append(&mut self, key: K, mut other: Self) {
        self.keys.push_back(key);
        self.keys.append(&mut other.keys);
        self.child.append(&mut other.child);
    }
    fn split_off(&mut self) -> (K, Self) {
        let child = if self.child.is_empty() {
            VecDeque::new()
        } else {
            self.child.split_off(ORDER)
        };
        let keys = self.keys.split_off(MIN_KEYS + 1);
        let mid = self.keys.pop_back().unwrap();
        (mid, Self { keys, child })
    }
}
fn linear_search<K: Ord>(v: &VecDeque<K>, key: &K) -> Result<usize, usize> {
    if let Some(i) = v.iter().position(|k| key <= k) {
        if v[i] == *key {
            Ok(i)
        } else {
            Err(i)
        }
    } else {
        Err(v.len())
    }
}

#[cfg(test)]
mod tests {
    use {super::BTree, rand::prelude::*, yansi::Paint};

    // -- unittest delete

    #[test]
    fn test_delete_from_leaf() {
        let mut test = Test::new();
        (0..4).for_each(|i| test.insert(i));
        test.delete(3);
    }

    #[test]
    fn test_no_delete() {
        let mut test = Test::new();
        (0..4).for_each(|i| test.insert(i));
        test.delete(4);
    }

    #[test]
    fn test_delete_from_narrow_leftmost_leaf() {
        let mut test = Test::new();
        (0..4).for_each(|i| test.insert(i));
        test.delete(0);
        test.delete(1);
    }

    #[test]
    fn test_delete_from_single_narrow_rightmost_leaf() {
        let mut test = Test::new();
        test.insert(0);
        test.insert(2);
        test.insert(3);
        test.insert(1);
        test.delete(3);
        test.delete(2);
    }

    #[test]
    fn test_delete_from_root() {
        let mut test = Test::new();
        (0..4).for_each(|i| test.insert(i));
        test.delete(1);
    }

    // -- random

    #[test]
    fn test_rand_small() {
        test_rand(100, 10, 42);
    }
    #[test]
    fn test_rand_large() {
        test_rand(10, 1000, 91);
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..t {
            let mut test = Test::new();
            for _ in 0..q {
                match rng.gen_range(0, 2) {
                    0 => test.insert(rng.gen_range(0, 200)),
                    1 => test.delete(rng.gen_range(0, 200)),
                    _ => unreachable!(),
                }
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
            println!("{}", Paint::red(format!("Insert {:?}", &x)).bold());
            self.bt.insert(x);
            if let Err(i) = self.vec.binary_search(&x) {
                self.vec.insert(i, x);
            }
            self.postprocess();
        }
        fn delete(&mut self, x: u32) {
            println!("{}", Paint::blue(format!("Delete {:?}", &x)).bold());
            self.bt.delete(x);
            if let Ok(i) = self.vec.binary_search(&x) {
                self.vec.remove(i);
            }
            self.postprocess();
        }
        fn postprocess(&self) {
            println!("paren = {:?}", super::paren::Wrapper(&self.bt));
            println!("bt = {:?}", &self.bt);
            assert_eq!(
                &self.bt.collect_vec().iter().copied().collect::<Vec<_>>(),
                &self.vec
            );
        }
    }
}
