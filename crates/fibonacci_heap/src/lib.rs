use {
    itertools::Itertools,
    paren::Paren,
    std::{
        cell::{Ref, RefCell},
        convert::identity,
        fmt::{self, Debug, Formatter},
        mem::{swap, take},
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
    pub fn push(&mut self, key: K) {
        self.chain.push(Rc::new(RefCell::new(Node::new(key))));
        if self.chain.first().unwrap().borrow().key > self.chain.last().unwrap().borrow().key {
            let len = self.chain.len();
            self.chain.swap(0, len - 1);
        }
        self.len += 1;
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
    key: K,
    child: Vec<Rc<RefCell<Node<K>>>>,
    parent: Weak<RefCell<Node<K>>>,
}
impl<K: Ord + Debug> Node<K> {
    pub fn new(key: K) -> Self {
        Self {
            key,
            child: Vec::new(),
            parent: Weak::new(),
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
impl<K: Ord + Debug> Paren for Node<K> {
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

#[cfg(test)]
mod tests {
    use {
        super::FibonacciHeap,
        paren::Paren,
        std::{
            cell::RefCell,
            cmp::Reverse,
            collections::BinaryHeap,
            fmt::Debug,
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
        fn push(&mut self, key: u32) {
            println!(
                "{} {} to {}",
                Paint::red("Push").bold(),
                key,
                self.fib.to_paren()
            );
            self.fib.push(key);
            self.bin.push(Reverse(key));
            self.postprocess();
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
}
