mod paren;

use std::{cmp::Ordering, fmt::Debug};

pub struct RBTree<K, V>(BoxedNode<K, V>);
impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    pub fn new() -> Self {
        Self(BoxedNode(None))
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v);
    }
    pub fn delete(&mut self, k: K) {
        self.0.delete(k);
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
        })))
    }
    fn is_nil(&self) -> bool {
        self.0.is_none()
    }

    fn replace(&mut self, x: Self) -> Self {
        std::mem::replace(self, x)
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

    // rb ops
    fn insert(&mut self, k: K, v: V) {
        if let Some(me) = self.0.as_mut() {
            let i = if k <= me.key { 0 } else { 1 };
            me.child[i].insert(k, v);
        } else {
            *self = Self::new(k, v);
        }
    }
    fn delete(&mut self, k: K) -> Option<BoxedNode<K, V>> {
        let me = self.0.as_mut()?;
        let i = match k.cmp(&me.key) {
            Ordering::Equal => {
                return Some(
                    if let Some(mut rem) = me.child[1].delete_first() {
                        (0..2).for_each(|i| rem.replace_empty_child(i, self.take_child(i)));
                        self.replace(rem)
                    } else {
                        assert!(self.child(1).is_nil());
                        let child = self.take_child(0);
                        self.replace(child)
                    }
                    .assert_isolated(),
                );
            }
            Ordering::Less => 0,
            Ordering::Greater => 1,
        };
        me.child[i].delete(k)
    }
    fn delete_first(&mut self) -> Option<BoxedNode<K, V>> {
        let me = self.0.as_mut()?;
        Some(
            if let Some(rem) = me.child[0].delete_first() {
                rem
            } else {
                assert!(self.child(0).is_nil());
                let child = self.take_child(1);
                self.replace(child)
            }
            .assert_isolated(),
        )
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
}

#[cfg(test)]
mod tests {
    use super::RBTree;
    use rand::prelude::*;

    #[test]
    fn test_rand_small() {
        test_rand(100, 20, 42);
        test_rand(100, 20, 43);
        test_rand(100, 20, 91);
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
                match rng.gen_range(0, 2) {
                    0 => test.insert(rng.gen_range(0, 10)),
                    1 => test.delete(rng.gen_range(0, 10)),
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
