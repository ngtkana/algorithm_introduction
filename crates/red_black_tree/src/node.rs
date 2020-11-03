use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    mem::replace,
    rc::{Rc, Weak},
};

pub enum Node<K, V> {
    Internal(Internal<K, V>),
    Nil(Nil<K, V>),
}
impl<K: Ord + Debug, V: Debug> Node<K, V> {
    pub fn as_internal(&self) -> Option<&Internal<K, V>> {
        match self {
            Node::Internal(internal) => Some(internal),
            Node::Nil(_) => None,
        }
    }
    pub fn as_internal_mut(&mut self) -> Option<&mut Internal<K, V>> {
        match self {
            Node::Internal(internal) => Some(internal),
            Node::Nil(_) => None,
        }
    }
    pub fn is_nil(&self) -> bool {
        match self {
            Node::Internal(_) => false,
            Node::Nil(_) => true,
        }
    }
    pub fn parent(&self) -> Option<&WeakNode<K, V>> {
        match self {
            Node::Internal(internal) => internal.parent.as_ref(),
            Node::Nil(nil) => nil.parent.as_ref(),
        }
    }
    pub fn take_parent(&mut self) -> Option<WeakNode<K, V>> {
        match self {
            Node::Internal(ref mut internal) => replace(&mut internal.parent, None),
            Node::Nil(ref mut nil) => replace(&mut nil.parent, None),
        }
    }
    pub fn replace_parent(&mut self, x: WeakNode<K, V>) -> Option<WeakNode<K, V>> {
        match self {
            Node::Internal(ref mut internal) => replace(&mut internal.parent, Some(x)),
            Node::Nil(ref mut nil) => replace(&mut nil.parent, Some(x)),
        }
    }
}

pub struct Internal<K, V> {
    child: [RcNode<K, V>; 2],
    parent: Option<WeakNode<K, V>>,
    key: K,
    value: V,
}
impl<K: Ord + Debug, V: Debug> Internal<K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }
    pub fn child(&self, i: usize) -> &RcNode<K, V> {
        &self.child[i]
    }
    pub fn take_child(&mut self, i: usize) -> RcNode<K, V> {
        let old = replace(&mut self.child[i], RcNode::nil());
        old
    }
    pub fn replace_child(&mut self, i: usize, x: RcNode<K, V>) -> RcNode<K, V> {
        let old = replace(&mut self.child[i], x);
        old
    }
}
pub struct Nil<K, V> {
    parent: Option<WeakNode<K, V>>,
}

// -- RcNode
pub struct RcNode<K, V>(Rc<RefCell<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> Clone for RcNode<K, V> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
impl<K: Ord + Debug, V: Debug> RcNode<K, V> {
    // -- constructor
    pub fn new(k: K, v: V) -> Self {
        let x = Self(rc_ref_cell(Node::Internal(Internal {
            parent: None,
            child: [Self::nil(), Self::nil()],
            key: k,
            value: v,
        })));
        let weak = Self::downgrade(&x);
        match &mut *x.as_mut() {
            Node::Internal(internal) => {
                internal.child[0]
                    .as_mut()
                    .replace_parent(WeakNode::clone(&weak));
                internal.child[1].as_mut().replace_parent(weak);
            }
            Node::Nil(_) => unreachable!(),
        }
        x
    }

    // -- observer
    pub fn index_parent(&self) -> Option<(usize, RcNode<K, V>)> {
        let self_mut = self.as_ref();
        self_mut.parent().map(|p| {
            let p: RcNode<K, V> = WeakNode::upgrade(p).unwrap();
            let i = match &*p.as_ref() {
                Node::Internal(internal) => (0..2)
                    .find(|&i| Self::ptr_eq(internal.child(i), self))
                    .unwrap(),
                Node::Nil(_) => panic!(),
            };
            (i, p)
        })
    }
    pub fn take_index_parent(&mut self) -> Option<(usize, RcNode<K, V>)> {
        let mut self_mut = self.as_mut();
        self_mut.take_parent().map(|p| {
            let p: RcNode<K, V> = WeakNode::upgrade(&p).unwrap();
            let i = match &*p.as_ref() {
                Node::Internal(internal) => (0..2)
                    .find(|&i| Self::ptr_eq(internal.child(i), self))
                    .unwrap(),
                Node::Nil(_) => panic!(),
            };
            (i, p)
        })
    }

    // -- ptr
    pub fn ptr_eq(x: &Self, y: &Self) -> bool {
        Rc::ptr_eq(&x.0, &y.0)
    }
    pub fn as_ref(&self) -> Ref<Node<K, V>> {
        self.0.borrow()
    }
    pub fn as_mut(&self) -> RefMut<Node<K, V>> {
        self.0.borrow_mut()
    }
    pub fn nil() -> Self {
        Self(rc_ref_cell(Node::Nil(Nil { parent: None })))
    }
    pub fn downgrade(&self) -> WeakNode<K, V> {
        WeakNode(Rc::downgrade(&self.0))
    }
    pub fn try_unwrap(self) -> Result<Option<(K, V)>, Self> {
        Rc::try_unwrap(self.0)
            .map_err(Self)
            .map(|node| match node.into_inner() {
                Node::Internal(internal) => Some((internal.key, internal.value)),
                Node::Nil(_) => None,
            })
    }

    // -- extrema
    pub fn tree_extremum(&self, i: usize) -> Self {
        let mut x = RcNode::clone(self);
        loop {
            let swp = match &*x.as_ref() {
                Node::Internal(internal) => RcNode::clone(internal.child(i)),
                Node::Nil(_) => break,
            };
            x = swp
        }
        x
    }
    pub fn tree_non_null_extremum(&self, i: usize) -> Option<Self> {
        let x = self.tree_extremum(i);
        if RcNode::ptr_eq(&x, self) {
            None
        } else {
            Some(WeakNode::upgrade(x.as_ref().parent().unwrap()).unwrap())
        }
    }

    // -- deformation
    pub fn connect(
        &mut self, /*Internal*/
        i: usize,
        x: &RcNode<K, V>,
    ) -> (RcNode<K, V>, Option<WeakNode<K, V>>) {
        match &mut *self.as_mut() {
            Node::Internal(ref mut internal) => {
                let old_child = internal.replace_child(i, RcNode::clone(x));
                let old_parent = x.as_mut().replace_parent(RcNode::downgrade(self));
                (old_child, old_parent)
            }
            Node::Nil(_) => panic!(),
        }
    }

    // -- collect
    pub fn collect_vec(&self, vec: &mut Vec<(K, V)>)
    where
        K: Clone,
        V: Clone,
    {
        match &*self.as_ref() {
            Node::Internal(internal) => {
                internal.child(0).collect_vec(vec);
                vec.push((internal.key.clone(), internal.value.clone()));
                internal.child(1).collect_vec(vec);
            }
            Node::Nil(_) => (),
        }
    }
}

pub struct WeakNode<K, V>(Weak<RefCell<Node<K, V>>>);
impl<K: Ord + Debug, V: Debug> WeakNode<K, V> {
    pub fn upgrade(x: &Self) -> Option<RcNode<K, V>> {
        Weak::upgrade(&x.0).map(RcNode)
    }
    pub fn ptr_eq(x: &Self, y: &Self) -> bool {
        Weak::ptr_eq(&x.0, &y.0)
    }
}
impl<K: Ord + Debug, V: Debug> Clone for WeakNode<K, V> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}
fn rc_ref_cell<T>(x: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(x))
}
