mod color;
mod node;
mod paren;
mod validate;

use color::Color;
use node::{Node, RcNode, WeakNode};
use std::{cmp::Ordering, fmt::Debug};

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
        let mut node = RcNode::new(k, v);
        self.transplant(&mut nil, &mut node);
        self.insert_fixup(node);
    }
    pub fn delete(&mut self, k: K) -> Option<RcNode<K, V>> {
        let mut found = self.find_node(&k)?;
        let mut child = found.clone_children().unwrap();
        let charged = if let Some(i) = child.iter().position(|child| child.is_nil()) {
            self.transplant(&mut found, &mut child[1 - i]);
            RcNode::clone(&child[1 - i])
        } else {
            let mut next = child[1].tree_non_null_extremum(0).unwrap();
            assert!(!RcNode::ptr_eq(&found, &next));
            let mut next1 = next.clone_child(1).unwrap();
            self.transplant(&mut next, &mut next1);
            self.transplant(&mut found, &mut next);
            next.swap_color(&mut found);
            let mut found_mut = found.as_mut();
            let found_internal = found_mut.as_internal_mut().unwrap();
            next.connect(0, &mut found_internal.take_child(0));
            next.connect(1, &mut found_internal.take_child(1));
            RcNode::clone(&next1)
        };
        if !self.root.is_nil() && found.is_black() {
            self.delete_fixup(charged);
        }
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
    fn insert_fixup(&mut self, mut x: RcNode<K, V>) {
        while !self.is_root(&x)
            && x.as_ref()
                .parent()
                .map_or(false, |p| WeakNode::upgrade(&p).unwrap().is_red())
        {
            let (i, mut p) = x.index_parent().unwrap();
            assert!(!p.is_nil());
            assert!(p.is_red());
            let (j, mut pp) = p.index_parent().unwrap();
            assert!(!pp.is_nil());
            assert!(pp.is_black());
            let mut y = pp.clone_child(1 - j).unwrap();
            if y.is_red() {
                p.set_color(Color::Black);
                y.set_color(Color::Black);
                pp.set_color(Color::Red);
                x = pp;
            } else if i == j {
                p.set_color(Color::Black);
                pp.set_color(Color::Red);
                self.rotate(&mut pp, j);
            } else {
                self.rotate(&mut p, i);
                x = p;
            }
        }
        self.root.set_color(Color::Black);
    }
    fn delete_fixup(&mut self, mut x: RcNode<K, V>) {
        while !self.is_root(&x) && x.is_black() {
            let (i, mut p) = x.index_parent().unwrap();
            assert!(!p.is_nil());
            let mut y = p.clone_child(1 - i).unwrap();
            if y.is_red() {
                p.swap_color(&mut y);
                self.rotate(&mut p, 1 - i);
            } else {
                assert!(y.is_black());
                assert!(!y.is_nil());
                let mut child = y.clone_children().unwrap();
                if child[0].is_black() && child[1].is_black() {
                    y.set_color(Color::Red);
                    x = p;
                } else if child[1 - i].is_black() {
                    assert!(child[i].is_red());
                    y.swap_color(&mut child[i]);
                    self.rotate(&mut y, i);
                } else {
                    p.swap_color(&mut y);
                    child[1 - i].set_color(Color::Black);
                    self.rotate(&mut p, 1 - i);
                    x = RcNode::clone(&self.root);
                }
            }
        }
        assert!(x.is_red() || self.is_root(&x));
        x.set_color(Color::Black);
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
    fn rotate(&mut self, x: &mut RcNode<K, V>, i: usize) {
        assert!(!x.is_nil());
        let mut y = x.clone_child(i).unwrap();
        assert!(!y.is_nil());
        let mut z = y.clone_child(1 - i).unwrap();
        if let Some((h, mut p)) = x.index_parent() {
            assert!(!self.is_root(&x));
            p.connect(h, &mut y);
        } else {
            assert!(self.is_root(&x));
            self.set_root(&mut y);
        }
        y.connect(1 - i, x);
        x.connect(i, &mut z);
    }
    fn transplant(&mut self, x: &mut RcNode<K, V>, y: &mut RcNode<K, V>) {
        let ip = x.take_index_parent();
        if let Some((i, mut p)) = ip {
            assert!(!RcNode::ptr_eq(&self.root, &x));
            let (old_child, _) = p.connect(i, y);
            assert!(RcNode::ptr_eq(&old_child, &x));
        } else {
            assert!(RcNode::ptr_eq(&self.root, &x));
            self.set_root(y);
        }
    }
    fn set_root(&mut self, x: &mut RcNode<K, V>) {
        x.take_parent();
        self.root = RcNode::clone(x);
    }
    fn is_root(&self, x: &RcNode<K, V>) -> bool {
        RcNode::ptr_eq(&self.root, x)
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
        println!("Insert {:?}.", key);
        println!("vec = {:?}.", &vec);
        println!("rbt = {:?}.", &rbt);
        println!();
        assert_eq!(
            &rbt.collect_vec()
                .into_iter()
                .map(|(k, _)| k)
                .collect::<Vec<_>>(),
            vec
        );
        Validate::all(rbt);
    }
    fn delete(key: u32, rbt: &mut RBTree<u32, ()>, vec: &mut Vec<u32>) {
        rbt.delete(key);
        let lb = vec.lower_bound(&key);
        if vec.get(lb).map_or(false, |x| x == &key) {
            vec.remove(lb);
        }
        println!("Delete {:?}.", key);
        println!("vec = {:?}.", &vec);
        println!("rbt = {:?}.", &rbt);
        println!();
        assert_eq!(
            &rbt.collect_vec()
                .into_iter()
                .map(|(k, _)| k)
                .collect::<Vec<_>>(),
            vec
        );
        Validate::all(rbt);
    }

    #[test]
    fn test_hand() {
        let mut rbt = RBTree::<u32, ()>::new();
        let mut vec = Vec::<u32>::new();

        insert(10, &mut rbt, &mut vec);
        insert(12, &mut rbt, &mut vec);
        insert(18, &mut rbt, &mut vec);
        insert(7, &mut rbt, &mut vec);
        insert(4, &mut rbt, &mut vec);
        insert(11, &mut rbt, &mut vec);

        delete(4, &mut rbt, &mut vec);
        delete(18, &mut rbt, &mut vec);
    }

    #[test]
    fn test_random() {
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..20 {
            let mut rbt = RBTree::<u32, ()>::new();
            let mut vec = Vec::<u32>::new();
            for _ in 0..2000 {
                match rng.gen_range(0, 2) {
                    0 => {
                        let key = rng.gen_range(0, 40);
                        insert(key, &mut rbt, &mut vec);
                    }
                    1 => {
                        let key = rng.gen_range(0, 40);
                        delete(key, &mut rbt, &mut vec);
                    }
                    _ => panic!(),
                }
            }
        }
    }
}
