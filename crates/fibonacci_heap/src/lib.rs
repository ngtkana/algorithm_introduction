use {
    itertools::Itertools,
    paren::Paren,
    std::{
        cell::{Ref, RefCell},
        fmt::{self, Debug, Formatter},
        mem::swap,
        rc::{Rc, Weak},
    },
};

pub struct FibonacciHeap<K>(Vec<Rc<RefCell<Node<K>>>>);
impl<K: Ord + Debug> FibonacciHeap<K> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, key: K) {
        self.0.push(Rc::new(RefCell::new(Node::new(key))));
        if self.0.first().unwrap().borrow().key > self.0.last().unwrap().borrow().key {
            let len = self.0.len();
            self.0.swap(0, len - 1);
        }
    }
    pub fn append(&mut self, other: &mut Self) {
        if let Some(me) = self.0.first_mut() {
            if let Some(other) = other.0.first_mut() {
                if me.borrow().key > other.borrow().key {
                    swap(&mut *me, &mut *other);
                }
            }
        }
        self.0.append(&mut other.0)
    }
    pub fn peek(&self) -> Option<Ref<K>> {
        self.0
            .first()
            .map(|node| Ref::map(node.borrow(), |node| &node.key))
    }
}

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
        self.0
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
        std::{cmp::Reverse, collections::BinaryHeap},
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
        fn postprocess(&self) {
            println!("fib = {}", self.fib.to_paren());
            println!("peek = {:?}", self.fib.peek());
            assert_eq!(
                self.fib.peek().map(|x| *x),
                self.bin.peek().map(|&Reverse(x)| x)
            );
            println!();
        }
    }
}
