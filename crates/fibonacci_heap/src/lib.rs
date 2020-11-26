use {
    itertools::Itertools,
    std::{
        cell::{Ref, RefCell},
        convert::identity,
        fmt::Debug,
        mem::{replace, swap, take},
        rc::{Rc, Weak},
    },
};

#[derive(Debug)]
pub struct FibonacciHeap<K> {
    len: usize,
    chain: Vec<Rc<RefCell<Node<K>>>>,
}
impl<K: Ord + Debug> FibonacciHeap<K> {
    pub fn new() -> Self {
        Self {
            len: 0,
            chain: Vec::new(),
        }
    }
    pub fn push(&mut self, key: K) -> Weak<RefCell<Node<K>>> {
        let handle = Rc::new(RefCell::new(Node::new(key)));
        self.chain.push(Rc::clone(&handle));
        if self.chain.first().unwrap().borrow().key > self.chain.last().unwrap().borrow().key {
            let len = self.chain.len();
            self.chain.swap(0, len - 1);
        }
        self.len += 1;
        Rc::downgrade(&handle)
    }
    pub fn append(&mut self, other: &mut Self) {
        self.len += other.len;
        if let Some(me) = self.chain.first_mut() {
            if let Some(other) = other.chain.first_mut() {
                if me.borrow().key > other.borrow().key {
                    swap(&mut *me, &mut *other);
                }
            }
        }
        self.chain.append(&mut other.chain)
    }
    pub fn peek(&self) -> Option<Ref<K>> {
        self.chain
            .first()
            .map(|node| Ref::map(node.borrow(), |node| &node.key))
    }
    pub fn pop(&mut self) -> Option<K> {
        if self.chain.is_empty() {
            None
        } else {
            self.len -= 1;
            let Node {
                mark: _mark,
                key,
                mut child,
                parent,
            } = Rc::try_unwrap(self.chain.swap_remove(0))
                .unwrap()
                .into_inner();
            assert!(parent.upgrade().is_none());
            self.chain.append(&mut child);
            self.consolidate();
            self.fix_top();
            Some(key)
        }
    }
    pub fn decrease_key(&mut self, x: Rc<RefCell<Node<K>>>, key: K) {
        assert!(
            key <= x.borrow().key,
            "A new key is greater than an old one: {:?} vs {:?}",
            &key,
            x.borrow().key
        );
        x.borrow_mut().key = key;
        let p = Weak::upgrade(&x.borrow().parent);
        if let Some(p) = p {
            if x.borrow().key < p.borrow().key {
                let x_pos = self.chain.len();
                self.cut(&p, x);
                let mut p = p;
                while replace(&mut p.borrow_mut().mark, true) {
                    let pp = Weak::upgrade(&p.borrow().parent);
                    p = if let Some(pp) = pp {
                        self.cut(&pp, p);
                        pp
                    } else {
                        break;
                    }
                }
                if self.chain[0].borrow().key > self.chain[x_pos].borrow().key {
                    self.chain.swap(0, x_pos);
                }
            }
        } else {
            self.fix_top();
        }
    }
    fn cut(&mut self, p: &Rc<RefCell<Node<K>>>, x: Rc<RefCell<Node<K>>>) {
        let i = p
            .borrow()
            .child
            .iter()
            .position(|node| Rc::ptr_eq(node, &x))
            .unwrap();
        let x = p.borrow_mut().child.swap_remove(i);
        x.borrow_mut().parent = Weak::new();
        x.borrow_mut().mark = false;
        self.chain.push(x);
    }

    fn consolidate(&mut self) {
        let n = self.len.next_power_of_two().trailing_zeros() as usize * 2;
        let mut a = vec![None::<Rc<RefCell<Node<K>>>>; n];
        let roots = take(&mut self.chain);
        for mut node in roots.into_iter() {
            loop {
                let len = node.borrow().child.len();
                if let Some(mut other) = a[len].take() {
                    let need_swap = node.borrow().key > other.borrow().key;
                    if need_swap {
                        swap(&mut node, &mut other);
                    }
                    other.borrow_mut().parent = Rc::downgrade(&node);
                    node.borrow_mut().child.push(other);
                } else {
                    break;
                }
            }
            let len = node.borrow().child.len();
            a[len] = Some(node);
        }
        self.chain = a.into_iter().filter_map(identity).collect();
    }
    fn fix_top(&mut self) {
        if let Some(i) = self
            .chain
            .iter()
            .position_min_by(|x, y| x.borrow().key.cmp(&y.borrow().key))
        {
            self.chain.swap(0, i);
        }
    }
}

#[derive(Debug)]
pub struct Node<K> {
    mark: bool,
    key: K,
    child: Vec<Rc<RefCell<Node<K>>>>,
    parent: Weak<RefCell<Node<K>>>,
}
impl<K: Ord + Debug> Node<K> {
    pub fn new(key: K) -> Self {
        Self {
            mark: false,
            key,
            child: Vec::new(),
            parent: Weak::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::FibonacciHeap,
        itertools::Itertools,
        paren::Paren,
        std::{
            cell::RefCell,
            cmp::Reverse,
            collections::BinaryHeap,
            fmt::{self, Debug, Formatter},
            rc::{Rc, Weak},
        },
        yansi::Paint,
    };

    #[test]
    fn test_push_append() {
        let mut test = Test::new();
        test.push(20);
        test.push(21);

        let mut other = Test::new();
        other.push(11);
        other.push(10);

        test.append(&mut other);
        test.push(30);
        other.push(40);
    }

    #[test]
    fn test_pop() {
        let mut test = Test::new();
        test.push(20);
        test.push(21);
        test.push(22);
        test.push(23);
        test.push(24);
        test.pop();
        test.pop();
        test.push(10);
        test.push(11);
        test.push(12);
        test.push(13);
        test.push(14);
        test.pop();
        test.pop();
        test.push(30);
        test.push(31);
        test.push(32);
        test.push(33);
        test.push(34);
        test.pop();
        test.pop();
    }

    #[test]
    fn test_decrease_key() {
        let mut test = Test::new();
        let h0 = test.push(20);
        test.decrease_key(Weak::upgrade(&h0).unwrap(), 18);
        let h1 = test.push(21);
        let h2 = test.push(22);
        let h3 = test.push(23);
        test.decrease_key(Weak::upgrade(&h1).unwrap(), 10);
        test.decrease_key(Weak::upgrade(&h2).unwrap(), 14);
        test.decrease_key(Weak::upgrade(&h3).unwrap(), 8);
        let h4 = test.push(24);
        test.pop();
        test.decrease_key(Weak::upgrade(&h1).unwrap(), 7);
        test.decrease_key(Weak::upgrade(&h2).unwrap(), 2);
        test.decrease_key(Weak::upgrade(&h4).unwrap(), 9);
    }

    struct Test {
        fib: FibonacciHeap<u32>,
        bin: BinaryHeap<Reverse<u32>>,
    }
    impl Test {
        fn new() -> Self {
            Self {
                fib: FibonacciHeap::new(),
                bin: BinaryHeap::new(),
            }
        }
        fn push(&mut self, key: u32) -> Weak<RefCell<super::Node<u32>>> {
            println!(
                "{} {} to {}",
                Paint::red("Push").bold(),
                key,
                self.fib.to_paren()
            );
            let res = self.fib.push(key);
            assert_eq!(Weak::upgrade(&res).unwrap().borrow().key, key);
            self.bin.push(Reverse(key));
            self.postprocess();
            res
        }
        fn append(&mut self, other: &mut Self) {
            println!(
                "{} {} to {}",
                Paint::magenta("Append").bold(),
                &other.fib.to_paren(),
                self.fib.to_paren()
            );
            self.fib.append(&mut other.fib);
            self.bin.extend(other.bin.drain());
            self.postprocess();
        }
        fn pop(&mut self) {
            println!("{} from {}", Paint::blue("Pop").bold(), self.fib.to_paren());
            let res = self.fib.pop();
            let exp = self.bin.pop().map(|x| x.0);
            assert_eq!(res, exp);
            self.postprocess();
        }
        fn decrease_key(&mut self, x: Rc<RefCell<super::Node<u32>>>, key: u32) {
            println!(
                "{} {} in {} down to {}",
                Paint::blue("Decrease a key").bold(),
                self.fib.to_paren(),
                &x.borrow().key,
                key,
            );
            let mut vec = self.bin.drain().collect::<Vec<_>>();
            let i = vec
                .iter()
                .position(|&Reverse(item)| item == x.borrow().key)
                .unwrap();
            vec[i] = Reverse(key);
            self.bin = vec.iter().copied().collect::<BinaryHeap<_>>();
            self.fib.decrease_key(x, key);
            self.postprocess();
        }
        fn postprocess(&self) {
            println!("{}", Paint::green("Postprocess"));
            println!("fib = {}", self.fib.to_paren());
            println!("peek = {:?}", self.fib.peek());
            assert_eq!(
                self.fib.peek().map(|x| *x),
                self.bin.peek().map(|&Reverse(x)| x)
            );
            self.fib.validate();
            println!();
        }
    }

    pub trait Validate {
        fn validate(&self);
    }
    impl<K: Ord + Debug> Validate for FibonacciHeap<K> {
        fn validate(&self) {
            self.chain.iter().for_each(|node| node.validate())
        }
    }
    impl<K: Ord + Debug> Validate for Rc<RefCell<super::Node<K>>> {
        fn validate(&self) {
            for child in self.borrow().child.iter() {
                assert!(
                    Weak::ptr_eq(&child.borrow().parent, &Rc::downgrade(self)),
                    "Parent of a child is not me."
                );
            }
        }
    }
    impl<K: Ord + Debug> Paren for FibonacciHeap<K> {
        fn paren(&self, w: &mut Formatter) -> fmt::Result {
            write!(w, "FibonacciHeap [")?;
            self.chain
                .iter()
                .map(|node| format!("{:?}", paren::Wrapper(&*node.borrow())))
                .intersperse(",".to_owned())
                .map(|s| write!(w, "{}", s))
                .collect::<fmt::Result>()?;
            write!(w, "]")
        }
    }
    impl<K: Ord + Debug> Paren for super::Node<K> {
        fn paren(&self, w: &mut Formatter) -> fmt::Result {
            write!(w, "(")?;
            write!(w, "{:?}", &self.key)?;
            self.child
                .iter()
                .map(|node| node.borrow().paren(w))
                .collect::<fmt::Result>()?;
            write!(w, ")")
        }
    }
}
