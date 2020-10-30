use std::{
    cell::RefCell,
    fmt,
    mem::{replace, swap},
    ops::Deref,
    rc::{Rc, Weak},
};
use yansi::Paint;

#[derive(Clone)]
pub struct RedBlackTree {
    root: Option<RcRefCell<Hook>>,
}
impl RedBlackTree {
    pub fn new() -> Self {
        Self { root: None }
    }
    pub fn insert(&mut self, key: u32) {
        if let Some(root) = self.root.as_ref() {
            let mut x = Rc::clone(root);
            let i = loop {
                let i = if key <= x.borrow().node.key { 0 } else { 1 };
                let y = x.borrow().child(i).map(Rc::clone);
                if let Some(y) = y.as_ref() {
                    x = Rc::clone(y);
                } else {
                    break i;
                }
            };
            let leaf = rc_ref_cell(Hook::red(Node { key }));
            Hook::connect(&x, i, Some(Rc::clone(&leaf)));
            if x.borrow().is_red() {
                self.fix_up(leaf);
            }
            return;
        }
        self.root = Some(rc_ref_cell(Hook::black(Node { key })));
    }
    /// `x` とその親が赤頂点のときに、なんとかします。
    fn fix_up(&mut self, mut x: RcRefCell<Hook>) {
        // safety: x は赤頂点なので親を持ちます。
        let (mut i, mut p) = Hook::index_parent(&x).unwrap();
        while p.borrow().is_red() {
            // safety: p は赤頂点なので親を持ちます。
            let (j, mut pp) = Hook::index_parent(&p).unwrap();
            assert!(pp.borrow().is_black());
            let y = pp.borrow().child(1 - j).map(Rc::clone);
            if let Some(y) = y.as_ref() {
                if y.borrow().is_red() {
                    p.borrow_mut().color = Color::Black;
                    y.borrow_mut().color = Color::Black;
                    pp.borrow_mut().color = Color::Red;
                    break;
                }
            }
            // safety: x は親 p を持ちます。
            if i != j {
                // 回転で i -> 1 - i とすべきですが、読まないので放置です。
                self.rotate(Rc::clone(&p), i);
                swap(&mut x, &mut p);
            }
            // 回転で j -> 1 - j とすべきですが、読むときに反転しているので放置です。
            if pp.borrow().is_black() {
                pp.borrow_mut().color = Color::Red;
                p.borrow_mut().color = Color::Black;
            }
            self.rotate(Rc::clone(&pp), j);
            swap(&mut p, &mut pp);
            x = replace(&mut p, pp);
            i = 1 - j;
        }
        // safety: `fix_up` がよばれるとき、根は存在します。
        self.root.as_mut().unwrap().borrow_mut().color = Color::Black;
        self.validate();
    }
    pub fn collect_vec(&self) -> Vec<u32> {
        let mut res = Vec::new();
        if let Some(root) = self.root.as_ref() {
            root.borrow().collect_vec(&mut res);
        }
        res
    }
    /// `i` 側の子を持ち上げます。
    fn rotate(&mut self, x: RcRefCell<Hook>, i: usize) {
        if let Some((h, p)) = Hook::index_parent(&x) {
            let y = Hook::rotate(Rc::clone(&x), i);
            Hook::connect(&p, h, Some(y));
        } else {
            let y = Hook::rotate(Rc::clone(&x), i);
            self.root = Some(y);
        }
    }
    fn validate(&self) {
        if let Some(root) = self.root.as_ref() {
            let ht = Hook::black_height(Rc::clone(root));
            Hook::validate(root, ht);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Node {
    key: u32,
}
#[derive(Debug, Clone, PartialEq, Copy)]
enum Color {
    Red,
    Black,
}
impl Color {
    fn paint<T>(self, t: T) -> Paint<T> {
        match self {
            Color::Red => Paint::red(t).bold(),
            Color::Black => Paint::blue(t).bold(),
        }
    }
}
#[derive(Clone)]
struct Hook {
    child: [Option<RcRefCell<Hook>>; 2],
    parent: Option<WeakRefCell<Hook>>,
    node: Node,
    color: Color,
}
impl Hook {
    fn red(node: Node) -> Self {
        Self {
            child: [None, None],
            parent: None,
            node,
            color: Color::Red,
        }
    }
    fn black(node: Node) -> Self {
        Self {
            child: [None, None],
            parent: None,
            node,
            color: Color::Black,
        }
    }
    fn is_red(&self) -> bool {
        matches!(self.color, Color::Red)
    }
    fn is_black(&self) -> bool {
        matches!(self.color, Color::Black)
    }
    fn parent(&self) -> Option<&WeakRefCell<Hook>> {
        self.parent.as_ref()
    }
    fn child(&self, i: usize) -> Option<&RcRefCell<Hook>> {
        self.child[i].as_ref()
    }
    fn index_parent(x: &RcRefCell<Self>) -> Option<(usize, RcRefCell<Hook>)> {
        // safety: 親が無効なケースはありません。
        let p = Weak::upgrade(x.borrow().parent()?).unwrap();
        let i = p
            .borrow()
            .child
            .iter()
            .position(|child| child.as_ref().map_or(false, |child| Rc::ptr_eq(child, x)))
            // safety: 親の子のいずれか一方は私です。
            .unwrap();
        Some((i, p))
    }
    fn connect(x: &RcRefCell<Self>, i: usize, y: Option<RcRefCell<Self>>) {
        if let Some(y) = y.as_ref() {
            y.borrow_mut().parent = Some(Rc::downgrade(x));
        }
        x.borrow_mut().child[i] = y;
    }
    fn take_child(x: &RcRefCell<Self>, i: usize) -> Option<RcRefCell<Self>> {
        let y = x.borrow_mut().child[i].take();
        y.as_ref().iter().for_each(|y| y.borrow_mut().parent = None);
        y
    }
    /// 回転をします。
    ///
    /// ### Requiries
    ///
    /// `x` は親を持ちません。（呼ぶ前に切り離してください）
    fn rotate(x: RcRefCell<Hook>, i: usize) -> RcRefCell<Hook> {
        let y = Hook::take_child(&x, i).unwrap();
        let z = Hook::take_child(&y, 1 - i);
        Hook::connect(&x, i, z);
        Hook::connect(&y, 1 - i, Some(x));
        y
    }
    fn collect_vec(&self, res: &mut Vec<u32>) {
        if let Some(child) = self.child(0) {
            child.borrow().collect_vec(res);
        }
        res.push(self.node.key);
        if let Some(child) = self.child(1) {
            child.borrow().collect_vec(res);
        }
    }
    fn validate(x: &RcRefCell<Self>, ht: u32) {
        let ht = match x.borrow().color {
            Color::Black => ht - 1,
            Color::Red => ht,
        };
        for i in 0..2 {
            if let Some(y) = x.borrow().child(i) {
                assert!(
                    x.borrow().is_black() || y.borrow().is_black(),
                    "i = {}, x = {:?}",
                    i,
                    x
                );
            } else {
                assert_eq!(ht, 0, "i = {}, x = {:?}", i, x);
            }
        }
    }
    fn black_height(mut x: RcRefCell<Hook>) -> u32 {
        let mut res = 0;
        if x.borrow().is_black() {
            res += 1;
        }
        loop {
            let y = x.borrow().child(0).map(Rc::clone);
            if let Some(y) = y {
                if y.borrow().is_black() {
                    res += 1;
                }
                x = Rc::clone(&y);
            } else {
                break res;
            }
        }
    }
}
type RcRefCell<T> = Rc<RefCell<T>>;
type WeakRefCell<T> = Weak<RefCell<T>>;
fn rc_ref_cell<T>(x: T) -> RcRefCell<T> {
    Rc::new(RefCell::new(x))
}

impl fmt::Debug for RedBlackTree {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "RedBlackTree ( ")?;
        if let Some(root) = self.root.as_ref() {
            write!(fmt, "{:?}", *root.borrow())?;
        }
        write!(fmt, " )")
    }
}
impl fmt::Debug for Hook {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(child) = self.child[0].as_ref() {
            write!(fmt, "({:?})", child.borrow().deref())?;
        }
        write!(fmt, "{:?}", self.color.paint(self.node.key))?;
        if let Some(child) = self.child[1].as_ref() {
            write!(fmt, "({:?})", child.borrow().deref())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{span::Span, RedBlackTree};
    use rand::prelude::*;

    #[test]
    fn test_hand() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..2000 {
            let mut bst = RedBlackTree::new();
            let mut vec = Vec::new();
            for _ in 0..20 {
                let key = rng.gen_range(0, 30);
                println!("insert {}", key);
                bst.insert(key);
                let lb = vec.lower_bound(&key);
                vec.insert(lb, key);
                println!("bst = {:?}", &bst);
                println!("vec = {:?}", &vec);
                assert_eq!(&vec, &bst.collect_vec());
            }
        }
    }
}
// span {{{
#[allow(dead_code)]
mod span {
    use std::{cmp, ops};

    impl<T> Span<T> for [T] {
        fn __span_internal_len(&self) -> usize {
            self.len()
        }

        fn __span_internal_is_empty(&self) -> bool {
            self.is_empty()
        }

        fn __span_internal_sort(&mut self)
        where
            T: cmp::Ord,
        {
            self.sort()
        }

        fn __span_internal_sort_by<F>(&mut self, compare: F)
        where
            F: FnMut(&T, &T) -> cmp::Ordering,
        {
            self.sort_by(compare)
        }

        fn __span_internal_sort_by_key<K, F>(&mut self, f: F)
        where
            F: FnMut(&T) -> K,
            K: cmp::Ord,
        {
            self.sort_by_key(f)
        }
    }

    pub trait Span<T>: ops::Index<usize, Output = T> {
        fn __span_internal_len(&self) -> usize;

        fn __span_internal_is_empty(&self) -> bool {
            self.__span_internal_len() == 0
        }

        fn __span_internal_sort(&mut self)
        where
            T: cmp::Ord;

        fn __span_internal_sort_by<F>(&mut self, compare: F)
        where
            F: FnMut(&T, &T) -> cmp::Ordering;

        fn __span_internal_sort_by_key<K, F>(&mut self, f: F)
        where
            F: FnMut(&T) -> K,
            K: cmp::Ord;

        fn sort_reverse(&mut self)
        where
            T: cmp::Ord,
        {
            self.__span_internal_sort_by(|a, b| a.cmp(b).reverse())
        }

        fn sort_reverse_by<F>(&mut self, mut compare: F)
        where
            F: FnMut(&T, &T) -> cmp::Ordering,
        {
            self.__span_internal_sort_by(|a, b| compare(a, b).reverse())
        }

        fn sort_reverse_by_key<K, F>(&mut self, mut f: F)
        where
            F: FnMut(&T) -> K,
            K: cmp::Ord,
        {
            self.__span_internal_sort_by_key(|x| cmp::Reverse(f(x)))
        }

        fn lower_bound<'a>(&'a self, x: &Self::Output) -> usize
        where
            T: Ord,
        {
            self.lower_bound_by(|p| p.cmp(x))
        }

        fn lower_bound_by_key<B, F>(&self, b: &B, mut f: F) -> usize
        where
            F: FnMut(&T) -> B,
            B: Ord,
        {
            self.lower_bound_by(|x| f(x).cmp(b))
        }

        fn lower_bound_by<F>(&self, mut f: F) -> usize
        where
            F: FnMut(&T) -> cmp::Ordering,
        {
            self.partition_point(|x| f(x) == cmp::Ordering::Less)
        }

        fn upper_bound<'a>(&'a self, x: &Self::Output) -> usize
        where
            Self::Output: Ord,
        {
            self.upper_bound_by(|p| p.cmp(x))
        }

        fn upper_bound_by_key<B, F>(&self, b: &B, mut f: F) -> usize
        where
            F: FnMut(&T) -> B,
            B: Ord,
        {
            self.upper_bound_by(|x| f(x).cmp(b))
        }

        fn upper_bound_by<F>(&self, mut f: F) -> usize
        where
            F: FnMut(&T) -> cmp::Ordering,
        {
            self.partition_point(|x| f(x) != cmp::Ordering::Greater)
        }

        fn partition_point<F>(&self, mut pred: F) -> usize
        where
            F: FnMut(&T) -> bool,
        {
            let mut left = 0;
            let mut right = self.__span_internal_len();
            while left != right {
                let mid = left + (right - left) / 2;
                let value = &self[mid];
                if pred(value) {
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }
            left
        }
    }
}
// }}}
// dbg {{{
#[allow(dead_code)]
mod dbg {
    #[macro_export]
    macro_rules! lg {
        () => {
            $crate::eprintln!("[{}:{}]", $crate::file!(), $crate::line!());
        };
        ($val:expr) => {
            match $val {
                tmp => {
                    eprintln!("[{}:{}] {} = {:?}",
                        file!(), line!(), stringify!($val), &tmp);
                    tmp
                }
            }
        };
        ($val:expr,) => { $crate::lg!($val) };
        ($($val:expr),+ $(,)?) => {
            ($($crate::lg!($val)),+,)
        };
    }

    #[macro_export]
    macro_rules! lg_nl {
        () => {
            eprintln!("[{}:{}]", $crate::file!(), $crate::line!());
        };
        ($val:expr) => {
            match $val {
                tmp => {
                    eprintln!("[{}:{}] {}:\n{:?}", file!(), line!(), stringify!($val), tmp);
                    tmp
                }
            };
        };
    }

    #[macro_export]
    macro_rules! msg {
        () => {
            compile_error!();
        };
        ($msg:expr) => {
            $crate::eprintln!("[{}:{}][{}]", $crate::file!(), $crate::line!(), $msg);
        };
        ($msg:expr, $val:expr) => {
            match $val {
                tmp => {
                    eprintln!("[{}:{}][{}] {} = {:?}",
                        file!(), line!(), $msg, stringify!($val), &tmp);
                    tmp
                }
            }
        };
        ($msg:expr, $val:expr,) => { msg!($msg, $val) };
        ($msg:expr, $($val:expr),+ $(,)?) => {
            ($(msg!($msg, $val)),+,)
        };
    }

    #[macro_export]
    macro_rules! tabular {
        ($val:expr) => {
            $crate::lg_nl!(crate::dbg::Tabular($val))
        };
    }

    #[macro_export]
    macro_rules! boolean_table {
        ($val:expr) => {
            $crate::lg_nl!(crate::dbg::BooleanTable($val));
        };
    }

    #[macro_export]
    macro_rules! boolean_slice {
        ($val:expr) => {
            $crate::lg!(crate::dbg::BooleanSlice($val));
        };
    }

    use std::fmt::{Debug, Formatter};

    #[derive(Clone)]
    pub struct Tabular<'a, T: Debug>(pub &'a [T]);
    impl<'a, T: Debug> Debug for Tabular<'a, T> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            for i in 0..self.0.len() {
                writeln!(f, "{:2} | {:?}", i, &self.0[i])?;
            }
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct BooleanTable<'a>(pub &'a [Vec<bool>]);
    impl<'a> Debug for BooleanTable<'a> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            for i in 0..self.0.len() {
                writeln!(f, "{:2} | {:?}", i, BooleanSlice(&self.0[i]))?;
            }
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct BooleanSlice<'a>(pub &'a [bool]);
    impl<'a> Debug for BooleanSlice<'a> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(
                f,
                "{}",
                self.0
                    .iter()
                    .map(|&b| if b { "1 " } else { "0 " })
                    .collect::<String>()
            )?;
            Ok(())
        }
    }
}
// }}}
