mod paren;
mod validate;

pub use validate::Validate;
use {
    std::{cmp::Ordering, fmt::Debug, iter::once, mem::replace, ops::Add},
    yansi::Paint,
};

pub struct LLRB<K, V>(BoxNode<K, V>);
impl<K: Ord + Debug, V: Clone + Add<Output = V> + Debug> LLRB<K, V> {
    pub fn new() -> Self {
        Self(BoxNode::nil())
    }
    pub fn len(&self) -> usize {
        self.0.size()
    }
    // FIXME: make `Node` private
    pub fn get(&self, i: usize) -> Option<&Node<K, V>> {
        self.0.get(i)
    }
    pub fn fold(&self, l: usize, r: usize) -> Option<V> {
        assert!(l <= r && r <= self.len());
        self.0.fold(l, r)
    }
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert(key, value);
        self.0.set_color(Color::Black);
    }
    pub fn delete(&mut self, key: &K) -> Option<(K, V)> {
        let root = &mut self.0;
        if root.is_two() {
            root.set_color(Color::Red)
        }
        let res = root
            .delete(key)
            .map(|node| node.0.unwrap())
            .map(|node| (node.key, node.value));
        if !root.is_nil() {
            root.set_color(Color::Black);
        }
        res
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
struct BoxNode<K, V>(Option<Box<Node<K, V>>>);
impl<K: Ord + Debug, V: Clone + Add<Output = V> + Debug> BoxNode<K, V> {
    fn nil() -> Self {
        Self(None)
    }
    fn is_nil(&self) -> bool {
        self.0.is_none()
    }
    fn size(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.size)
    }
    fn acc(&self) -> Option<&V> {
        self.0.as_ref().map(|node| &node.acc)
    }
    fn update(&mut self) {
        println!("update {:?}", &self);
        if let Some(me) = self.0.as_mut() {
            me.size = 1 + me.child.iter().map(|child| child.size()).sum::<usize>();
            me.acc = me
                .child
                .iter()
                .map(|child| child.acc())
                .flatten()
                .fold(me.value.clone(), |x, y| x + y.clone())
        }
    }
    fn get(&self, i: usize) -> Option<&Node<K, V>> {
        let me = self.0.as_ref()?;
        let lsize = me.child[0].size();
        match i.cmp(&lsize) {
            Ordering::Less => me.child[0].get(i),
            Ordering::Greater => me.child[1].get(i - lsize - 1),
            Ordering::Equal => Some(me),
        }
    }
    fn fold(&self, l: usize, r: usize) -> Option<V> {
        if l == r {
            None
        } else {
            let me = self.0.as_ref()?;
            assert!(l <= r && r <= me.size);
            if (l, r) == (0, me.size) {
                Some(me.acc.clone())
            } else {
                let lsize = me.child[0].size();
                once(me.child[0].fold(l.min(lsize), r.min(lsize)))
                    .chain(once(if l <= lsize && lsize < r {
                        Some(me.value.clone())
                    } else {
                        None
                    }))
                    .chain(once(me.child[1].fold(
                        l.saturating_sub(lsize + 1),
                        r.saturating_sub(lsize + 1),
                    )))
                    .fold(None, |x, y| match (x, y) {
                        (None, None) => None,
                        (Some(x), None) => Some(x),
                        (None, Some(y)) => Some(y),
                        (Some(x), Some(y)) => Some(x + y),
                    })
            }
        }
    }
    fn insert(&mut self, key: K, value: V) {
        if self.is_nil() {
            *self = Node::new(key, value, Color::Red).boxed();
        } else {
            self.child_mut(match key.cmp(&self.unwrap().key) {
                Ordering::Less => 0,
                Ordering::Greater => 1,
                Ordering::Equal => return,
            })
            .insert(key, value);
            self.fixup();
        }
    }
    fn delete(&mut self, key: &K) -> Option<Self> {
        if self.is_nil() {
            None
        } else {
            let cmp = key.cmp(&self.unwrap().key);
            let rem = if cmp == Ordering::Less {
                // Merge 2-nodes
                if self.child(0).is_two() {
                    self.move_left();
                }
                self.child_mut(0).delete(key)
            } else {
                // Lean right
                if self.child(0).is_red() {
                    self.rotate(0);
                }
                if cmp == Ordering::Equal && self.child(1).is_nil() {
                    return Some(replace(self, Self::nil()));
                }
                // Merge 2-nodes
                if self.child(1).is_two() {
                    self.move_right();
                }
                if key == &self.unwrap().key {
                    let mut rem = self.child_mut(1).delete_first();
                    (0..2).for_each(|i| rem.init_child(i, self.take_child(i)));
                    rem.set_color(self.color());
                    Some(replace(self, rem))
                } else {
                    self.child_mut(1).delete(key)
                }
            };
            self.fixup();
            rem
        }
    }
    fn delete_first(&mut self) -> Self {
        if self.child(0).is_nil() {
            replace(self, Self::nil())
        } else {
            // Merge 2-nodes
            if self.child(0).is_two() {
                self.move_left();
            }
            let rem = self.child_mut(0).delete_first();
            self.fixup();
            rem
        }
    }
    fn fixup(&mut self) {
        self.update();
        // Balance right-leaning red
        if self.child(0).is_black() && self.child(1).is_red() {
            self.rotate(1);
        }
        // Balance double red
        if self.child(0).is_red() && self.child(0).child(0).is_red() {
            self.rotate(0);
        }
        // Split 4-nodes
        if self.child(0).is_red() && self.child(1).is_red() {
            self.split_node();
        }
        self.update();
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

    // -- color
    fn color(&self) -> Color {
        self.0.as_ref().map_or(Color::Black, |node| node.color)
    }
    fn is_red(&self) -> bool {
        self.color() == Color::Red
    }
    fn is_black(&self) -> bool {
        self.color() == Color::Black
    }

    // -- unwrap
    fn unwrap(&self) -> &Node<K, V> {
        self.0.as_ref().unwrap()
    }
    fn unwrap_mut(&mut self) -> &mut Node<K, V> {
        self.0.as_mut().unwrap()
    }
    fn set_color(&mut self, color: Color) {
        self.unwrap_mut().color = color
    }
    fn is_two(&self) -> bool {
        !self.is_nil() && self.is_black() && self.child(0).is_black() && self.child(1).is_black()
    }

    // -- child
    fn child(&self, i: usize) -> &Self {
        &self.unwrap().child[i]
    }
    fn child_mut(&mut self, i: usize) -> &mut Self {
        &mut self.unwrap_mut().child[i]
    }
    fn take_child(&mut self, i: usize) -> Self {
        replace(&mut self.unwrap_mut().child[i], Self::nil())
    }
    fn init_child(&mut self, i: usize, x: Self) {
        let old = replace(&mut self.unwrap_mut().child[i], x);
        assert!(old.is_nil());
    }

    // -- rotate
    fn rotate(&mut self, i: usize) {
        let mut x = replace(self, Self::nil());
        let mut y = x.take_child(i);
        assert!(y.is_red());
        y.set_color(x.color());
        x.set_color(Color::Red);
        let z = y.take_child(1 - i);
        x.init_child(i, z);
        x.update();
        y.init_child(1 - i, x);
        y.update();
        *self = y;
    }
    fn move_left(&mut self) {
        assert!(self.is_red());
        assert!(self.child(0).is_two());
        self.merge_node();
        if self.child(1).child(0).is_red() {
            self.child_mut(1).rotate(0);
            self.rotate(1);
            self.split_node();
        }
    }
    fn move_right(&mut self) {
        assert!(self.is_red());
        assert!(self.child(1).is_two());
        self.merge_node();
        if self.child(0).child(0).is_red() {
            self.rotate(0);
            self.child_mut(1).rotate(1);
            self.split_node();
        }
    }

    fn merge_node(&mut self) {
        assert!(self.is_red());
        (0..2).for_each(|i| assert!(self.child(i).is_black()));
        self.set_color(Color::Black);
        (0..2).for_each(|i| self.child_mut(i).set_color(Color::Red));
    }
    fn split_node(&mut self) {
        assert!(self.is_black());
        (0..2).for_each(|i| assert!(self.child(i).is_red()));
        self.set_color(Color::Red);
        (0..2).for_each(|i| self.child_mut(i).set_color(Color::Black));
    }
}
pub struct Node<K, V> {
    child: [BoxNode<K, V>; 2],
    key: K,
    value: V,
    acc: V,
    color: Color,
    size: usize,
}
impl<K: Ord + Debug, V: Clone + Add<Output = V> + Debug> Node<K, V> {
    fn new(key: K, value: V, color: Color) -> Self {
        Self {
            child: [BoxNode::nil(), BoxNode::nil()],
            key,
            acc: value.clone(),
            value,
            color,
            size: 1,
        }
    }
    fn boxed(self) -> BoxNode<K, V> {
        BoxNode(Some(Box::new(self)))
    }
}
#[derive(Debug, Clone, PartialEq, Copy, Eq)]
enum Color {
    Red,
    Black,
}
impl Color {
    fn paint<T>(&self, x: T) -> Paint<T> {
        match self {
            Self::Red => Paint::red(x),
            Self::Black => Paint::blue(x),
        }
        .bold()
    }
}

#[cfg(test)]
mod tests {
    use super::Validate;
    use super::LLRB;
    use rand::prelude::*;
    use randtools;

    #[test]
    fn test_hand_insert() {
        let mut test = Test::new();
        test.insert(10, 10);
        test.insert(11, 10);
        test.insert(18, 10);
        test.insert(14, 10);
        test.insert(13, 10);
        test.insert(18, 10);
    }

    #[test]
    fn test_hand_delete() {
        let mut test = Test::new();
        test.insert(10, 10);
        test.insert(11, 10);
        test.insert(15, 10);
        test.insert(17, 10);
        test.delete(17);
        test.delete(11);
        test.delete(12);
    }

    #[test]
    fn test_hand_get() {
        let mut test = Test::new();
        test.insert(10, 10);
        test.insert(11, 10);
        test.insert(18, 10);
        test.insert(14, 10);
        test.insert(13, 10);
        test.insert(18, 10);
        (0..10).for_each(|i| test.get(i));
    }

    #[test]
    fn test_rand_small() {
        test_rand(10, 50, 42);
        test_rand(10, 50, 91);
    }

    #[test]
    fn test_rand_large() {
        test_rand(10, 200, 42);
        test_rand(10, 200, 91);
    }

    #[test]
    fn test_oneline() {
        let mut test = Test::new();
        (0..100).for_each(|i| test.insert(i, 10));
        (0..100).for_each(|i| test.delete(i));
    }

    #[test]
    fn test_oneline_reverse() {
        let mut test = Test::new();
        (0..100).for_each(|i| test.insert(i, 10));
        (0..100).rev().for_each(|i| test.delete(i));
    }

    fn test_rand(t: u32, q: u32, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        for i in 0..t {
            let mut test = Test::new();
            for j in 0..q {
                println!("Test {}, Query {}", i, j);
                match rng.gen_range(0, 4) {
                    0 => test.insert(rng.gen_range(0, 30), rng.gen_range(0, 8)),
                    1 => test.delete(rng.gen_range(0, 30)),
                    2 => {
                        if test.len() == 0 {
                            println!("Skiped `get`");
                        } else {
                            test.get(rng.gen_range(0, test.len()));
                        }
                    }
                    3 => {
                        if test.len() == 0 {
                            println!("Skiped `fold`");
                        } else {
                            let (l, r) = randtools::Open(0, test.len()).sample(&mut rng);
                            test.fold(l, r);
                        }
                    }
                    _ => panic!(),
                }
            }
        }
    }

    struct Test {
        llrb: LLRB<u32, u32>,
        vec: Vec<(u32, u32)>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                llrb: LLRB::new(),
                vec: Vec::new(),
            }
        }
        fn len(&self) -> usize {
            self.vec.len()
        }
        fn insert(&mut self, key: u32, value: u32) {
            println!("Insert {:?}", &key);
            self.llrb.insert(key, value);
            println!("llrb = {:?}", &self.llrb);
            if let Err(i) = self.vec.binary_search_by_key(&key, |&(key, _)| key) {
                self.vec.insert(i, (key, value));
            }
            self.postprocess();
        }
        fn delete(&mut self, x: u32) {
            println!("Delete {:?}", &x);
            self.llrb.delete(&x);
            println!("llrb = {:?}", &self.llrb);
            if let Ok(i) = self.vec.binary_search_by_key(&x, |&(key, _)| key) {
                self.vec.remove(i);
            }
            self.postprocess();
        }
        fn get(&mut self, i: usize) {
            println!("Get {:?}", &i);
            let result = self.llrb.get(i).map(|node| (node.key, node.value));
            let expected = self.vec.get(i).copied();
            println!("result = {:?}, expected = {:?}", result, expected);
            assert_eq!(result, expected, "Failed in `get`");
            self.postprocess();
        }
        fn fold(&mut self, l: usize, r: usize) {
            println!("Fold {}..{}", l, r);
            let result = self.llrb.fold(l, r).unwrap_or(0);
            let expected = self.vec[l..r].iter().map(|(_, value)| value).sum::<u32>();
            println!("result = {:?}, expected = {:?}", result, expected);
            assert_eq!(result, expected, "Failed in `fold`");
            self.postprocess();
        }
        fn postprocess(&self) {
            Validate::validate(&self.llrb);
            assert_eq!(&self.llrb.collect_vec(), &self.vec);
            assert_eq!(self.llrb.len(), self.vec.len());
        }
    }
}
