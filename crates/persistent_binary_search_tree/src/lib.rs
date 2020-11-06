mod paren;

use std::{fmt::Debug, rc::Rc};

pub struct PBST<K, V>(Vec<RcNode<K, V>>);
impl<K: Ord + Debug, V: Debug> PBST<K, V> {
    pub fn new() -> Self {
        Self(vec![RcNode(None)])
    }
    pub fn insert(&mut self, k: K, v: V) {
        let root = self.0.last().unwrap().insert(k, v);
        self.0.push(root);
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

struct RcNode<K, V>(Option<Rc<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> Clone for RcNode<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.as_ref().map(Rc::clone))
    }
}
impl<K: Ord + Debug, V: Debug> RcNode<K, V> {
    fn new(k: K, v: V) -> Self {
        Self(Some(Rc::new(Node {
            child: [Self(None), Self(None)],
            kv: Rc::new((k, v)),
        })))
    }
    fn insert(&self, k: K, v: V) -> RcNode<K, V> {
        if let Some(internal) = self.0.as_ref() {
            let i = if k <= internal.kv.0 { 0 } else { 1 };
            let new_child = internal.child[i].insert(k, v);
            let mut me = Node {
                child: [RcNode(None), RcNode(None)],
                kv: Rc::clone(&self.0.as_ref().unwrap().kv),
            };
            me.child[i] = new_child;
            me.child[1 - i] = RcNode::clone(&internal.child[1 - i]);
            Self(Some(Rc::new(me)))
        } else {
            Self::new(k, v)
        }
    }
    fn collect_vec(&self, vec: &mut Vec<(K, V)>)
    where
        K: Clone,
        V: Clone,
    {
        if let Some(internal) = self.0.as_ref() {
            internal.child[0].collect_vec(vec);
            vec.push((*internal.kv).clone());
            internal.child[1].collect_vec(vec);
        }
    }
}

struct Node<K, V> {
    child: [RcNode<K, V>; 2],
    kv: Rc<(K, V)>,
}

#[cfg(test)]
mod tests {
    use super::PBST;

    #[test]
    fn test_hand() {
        let mut test = Test::new();

        test.insert(10);
        test.insert(12);
        test.insert(11);
        test.insert(9);
        test.insert(12);
        test.insert(10);
        test.insert(10);
        test.insert(7);
        test.insert(17);
        test.insert(14);
        test.insert(6);
    }

    struct Test {
        time: u32,
        pbst: PBST<u32, ()>,
        vec: Vec<Vec<u32>>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                time: 0,
                pbst: PBST::new(),
                vec: vec![Vec::new()],
            }
        }
        fn increment(&mut self) {
            self.time += 1;
        }
        fn insert(&mut self, k: u32) {
            println!("Insert {:?}", k);

            self.pbst.insert(k, ());
            let mut v = self.vec.last().unwrap().clone();
            let lb = v.binary_search(&k).map_or_else(|e| e, |x| x);
            v.insert(lb, k);
            self.vec.push(v);

            println!(
                "rbst[{}] = {:?}",
                self.time, &self.pbst.0[self.time as usize]
            );

            self.increment();
            for i in 0..self.time as usize {
                let result = self
                    .pbst
                    .collect_vec(i)
                    .iter()
                    .map(|&(k, ())| k)
                    .collect::<Vec<_>>();
                let expected = self.vec[i].clone();
                assert_eq!(result, expected, "Time = {}/{}", i, self.time);
            }
        }
    }
}
