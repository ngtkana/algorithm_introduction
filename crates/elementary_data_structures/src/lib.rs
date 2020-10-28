use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct LinkedList {
    first: Option<Rc<RefCell<Hook>>>,
}
impl LinkedList {
    pub fn new() -> Self {
        Self { first: None }
    }
    pub fn push_front(&mut self, key: u32) {
        let mut new_first = Rc::new(RefCell::new(Hook::new(key)));
        if let Some(orig_first) = self.first.as_mut() {
            Hook::link(&mut new_first, orig_first);
            *orig_first = new_first;
        } else {
            self.first = Some(new_first);
        }
    }
    pub fn pop_front(&mut self) -> Option<Node> {
        if let Some(first) = self.first.take() {
            let second = first.borrow_mut().split_next();
            let hook = Rc::try_unwrap(first).unwrap().into_inner();
            self.first = second;
            Some(hook.into_node())
        } else {
            None
        }
    }
    pub fn contains(&mut self, k: u32) -> bool {
        self.first
            .as_ref()
            .map(Rc::clone)
            .and_then(|mut node| loop {
                if node.borrow().node.key == k {
                    break Some(());
                } else if let Some(next) = Rc::clone(&node).borrow().next.as_ref().map(Rc::clone) {
                    node = next;
                } else {
                    break None;
                }
            })
            .is_some()
    }
    pub fn delete(&mut self, k: u32) -> Option<Node> {
        if self
            .first
            .as_ref()
            .map_or(false, |first| first.borrow().node.key == k)
        {
            self.pop_front()
        } else {
            self.first
                .as_ref()
                .map(Rc::clone)
                .and_then(|mut node| loop {
                    if node.borrow().node.key == k {
                        node.borrow().escape();
                        break Some(Rc::try_unwrap(node).unwrap().into_inner().into_node());
                    } else if let Some(next) =
                        Rc::clone(&node).borrow().next.as_ref().map(Rc::clone)
                    {
                        node = next;
                    } else {
                        break None;
                    }
                })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hook {
    node: Node,
    next: Option<Rc<RefCell<Hook>>>,
    prev: Option<Weak<RefCell<Hook>>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    key: u32,
}
impl Hook {
    pub fn new(key: u32) -> Self {
        Self {
            node: Node { key },
            next: None,
            prev: None,
        }
    }
    pub fn into_node(self) -> Node {
        self.node
    }
    pub fn split_next(&mut self) -> Option<Rc<RefCell<Hook>>> {
        self.next.take().map(|next| {
            next.borrow_mut().prev = None;
            next
        })
    }
    pub fn escape(&self) {
        match (&self.prev, &self.next) {
            (Some(prev), Some(next)) => {
                prev.upgrade().unwrap().borrow_mut().next = Some(Rc::clone(next));
                next.borrow_mut().prev = Some(Weak::clone(prev));
            }
            (Some(prev), None) => {
                prev.upgrade().unwrap().borrow_mut().next = None;
            }
            (None, Some(next)) => {
                next.borrow_mut().prev = None;
            }
            (None, None) => (),
        }
    }
    pub fn link(former: &mut Rc<RefCell<Hook>>, latter: &mut Rc<RefCell<Hook>>) {
        assert!(former.borrow().next.is_none());
        assert!(latter.borrow().prev.is_none());
        former.borrow_mut().next = Some(Rc::clone(latter));
        latter.borrow_mut().prev = Some(Rc::downgrade(former));
    }
}
impl Drop for Node {
    fn drop(&mut self) {
        println!("dropped!: {}", self.key);
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "[")?;
        let mut node = self.first.as_ref().map(Rc::clone);
        while let Some(n) = node {
            write!(w, "{}({:?})", n.borrow().node.key, n.as_ptr())?;
            node = n.borrow().next.as_ref().map(Rc::clone);
            if node.is_some() {
                write!(w, ", ")?;
            }
        }
        write!(w, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::LinkedList;

    #[test]
    fn test_linked_list() {
        let mut list = LinkedList::new();
        println!("{:?}", &list);
        for i in 0..4 {
            list.push_front(i);
        }
        println!("{:?}", &list);

        println!("popped: {:?}", list.pop_front());
        println!("{:?}", &list);

        println!("Search: {:?}", list.contains(0));
        println!("Search: {:?}", list.contains(1));
        println!("Search: {:?}", list.contains(2));
        println!("Search: {:?}", list.contains(5));
        println!("{:?}", &list);

        println!("Delete: {:?}", list.delete(0));
        println!("{:?}", &list);
        println!("Delete: {:?}", list.delete(1));
        println!("{:?}", &list);
        println!("Delete: {:?}", list.delete(2));
        println!("{:?}", &list);
        println!("Delete: {:?}", list.delete(5));
        println!("{:?}", &list);
    }
}
