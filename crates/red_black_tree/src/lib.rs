mod node;
mod paren;
mod validate;

use dbg::{lg, msg};
use node::{Node, RcNode};
use std::{cmp::Ordering, fmt::Debug};
use validate::Validate;

pub struct RBTree<K, V> {
    root: RcNode<K, V>,
}
impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: RcNode::nil(),
        }
    }
    pub fn insert(&mut self, k: K, v: V) {
        let mut nil = self.find_insert_position(&k);
        let node = RcNode::new(k, v);
        self.transplant(&mut nil, &node);
    }
    pub fn delete(&mut self, k: K) -> Option<RcNode<K, V>> {
        msg!("delete", (&self, &k));
        let mut found = self.find_node(&k)?;
        let child = {
            let found_ref = found.as_ref();
            let found_internal = found_ref.as_internal().unwrap();
            [
                RcNode::clone(found_internal.child(0)),
                RcNode::clone(found_internal.child(1)),
            ]
        };
        msg!("delete", (&self, &k));
        let next = if let Some(i) = child.iter().position(|child| child.as_ref().is_nil()) {
            self.transplant(&mut found, &child[1 - i]);
            RcNode::clone(&found)
        } else {
            let mut next = child[1].tree_non_null_extremum(0).unwrap();
            assert!(!RcNode::ptr_eq(&found, &next));
            let next1 = {
                let next_ref = next.as_ref();
                RcNode::clone(&next_ref.as_internal().unwrap().child(1))
            };
            msg!("before", &self, &found, &next);
            self.transplant(&mut next, &next1);
            self.transplant(&mut found, &next);
            let mut found_mut = found.as_mut();
            let found_internal = found_mut.as_internal_mut().unwrap();
            next.connect(0, &found_internal.take_child(0));
            next.connect(1, &found_internal.take_child(1));
            RcNode::clone(&next)
        };
        msg!("after", &self, &found, &next);
        Validate::reflexive_parent(self);
        Some(RcNode::clone(&found))
    }
    pub fn collect_vec(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut vec = Vec::new();
        self.root.collect_vec(&mut vec);
        vec
    }
    fn find_node(&self, k: &K) -> Option<RcNode<K, V>> {
        let mut x = RcNode::clone(&self.root);
        loop {
            let swp = match &*x.as_ref() {
                Node::Internal(internal) => {
                    RcNode::clone(internal.child(match k.cmp(internal.key()) {
                        Ordering::Equal => return Some(RcNode::clone(&x)), // TODO: move したいです。
                        Ordering::Less => 0,
                        Ordering::Greater => 1,
                    }))
                }
                Node::Nil(_) => return None,
            };
            x = swp;
        }
    }
    fn find_insert_position(&self, k: &K) -> RcNode<K, V> {
        let mut x = RcNode::clone(&self.root);
        loop {
            let swp = match &*x.as_ref() {
                Node::Internal(internal) => {
                    RcNode::clone(internal.child(if k <= internal.key() { 0 } else { 1 }))
                }
                Node::Nil(_) => break,
            };
            x = swp;
        }
        x
    }
    fn transplant(&mut self, x: &mut RcNode<K, V>, y: &RcNode<K, V>) {
        let ip = x.take_index_parent();
        if let Some((i, mut p)) = ip {
            assert!(!RcNode::ptr_eq(&self.root, &x));
            let (old_child, _) = p.connect(i, y);
            assert!(RcNode::ptr_eq(&old_child, &x));
        } else {
            assert!(RcNode::ptr_eq(&self.root, &x));
            let mut y_mut = y.as_mut();
            y_mut.take_parent();
            self.root = RcNode::clone(y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate::Validate;
    use super::RBTree;
    use rand::prelude::*;
    use span::Span;

    fn insert(key: u32, rbt: &mut RBTree<u32, ()>, vec: &mut Vec<u32>) {
        rbt.insert(key, ());
        let lb = vec.lower_bound(&key);
        vec.insert(lb, key);
        println!("- Insert {:?}.", key);
        println!("- rbt = {:?}.", &rbt);
        println!("- vec = {:?}.", &vec);
        println!();
        assert_eq!(
            &rbt.collect_vec()
                .into_iter()
                .map(|(k, _)| k)
                .collect::<Vec<_>>(),
            vec
        );
        Validate::reflexive_parent(rbt);
    }
    fn delete(key: u32, rbt: &mut RBTree<u32, ()>, vec: &mut Vec<u32>) {
        rbt.delete(key);
        let lb = vec.lower_bound(&key);
        if vec.get(lb).map_or(false, |x| x == &key) {
            vec.remove(lb);
        }
        println!("- Delete {:?}.", key);
        println!("- rbt = {:?}.", &rbt);
        println!("- vec = {:?}.", &vec);
        println!();
        assert_eq!(
            &rbt.collect_vec()
                .into_iter()
                .map(|(k, _)| k)
                .collect::<Vec<_>>(),
            vec
        );
        Validate::reflexive_parent(rbt);
    }

    #[test]
    fn test_hand() {
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..20 {
            let mut rbt = RBTree::<u32, ()>::new();
            let mut vec = Vec::<u32>::new();
            for _ in 0..20 {
                match rng.gen_range(0, 2) {
                    0 => {
                        let key = rng.gen_range(0, 10);
                        insert(key, &mut rbt, &mut vec);
                    }
                    1 => {
                        let key = rng.gen_range(0, 10);
                        delete(key, &mut rbt, &mut vec);
                    }
                    _ => panic!(),
                }
            }
        }
    }
}
