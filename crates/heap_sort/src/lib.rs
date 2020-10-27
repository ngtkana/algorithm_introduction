#[derive(Debug, Clone, PartialEq)]
pub struct BinaryHeap {
    heap_size: usize,
    a: Vec<u32>,
}

impl BinaryHeap {
    pub fn parent(i: usize) -> usize {
        i / 2
    }
    pub fn left(i: usize) -> usize {
        2 * i
    }
    pub fn right(i: usize) -> usize {
        2 * i + 1
    }
    pub fn max_heapify(&mut self, i: usize) {
        let l = Self::left(i);
        let r = Self::right(i);
        let mut largest = if l <= self.heap_size && self.a[l] > self.a[i] {
            l
        } else {
            i
        };
        if r <= self.heap_size && self.a[r] > self.a[largest] {
            largest = r;
        }
        if largest != i {
            self.a.swap(i, largest);
            self.max_heapify(largest);
        }
    }
    pub fn build(a: &[u32]) -> Self {
        let mut me = Self {
            heap_size: a.len(),
            a: std::iter::once(0).chain(a.iter().copied()).collect(),
        };
        me.build_max();
        me
    }
    pub fn build_max(&mut self) {
        self.heap_size = self.a.len() - 1;
        for i in (1..=self.a.len() / 2).rev() {
            self.max_heapify(i);
        }
    }
    pub fn maximum(&self) -> u32 {
        self.a[1]
    }
    pub fn extract_max(&mut self) -> u32 {
        assert!(self.heap_size != 0);
        let max = self.a[1];
        self.a[1] = self.a[self.heap_size];
        self.heap_size -= 1;
        self.max_heapify(1);
        return max;
    }
    pub fn increase_key(&mut self, mut i: usize, key: u32) {
        assert!(self.a[i] <= key);
        self.a[i] = key;
        while i > 1 && self.a[Self::parent(i)] < self.a[i] {
            self.a.swap(i, Self::parent(i));
            i = Self::parent(i);
        }
    }
    pub fn insert(&mut self, key: u32) {
        self.heap_size += 1;
        if self.heap_size == self.a.len() {
            self.a.push(0);
        }
        self.a[self.heap_size] = std::u32::MIN;
        self.increase_key(self.heap_size, key);
    }
    pub fn sort(&mut self) {
        self.build_max();
        for i in (2..self.a.len()).rev() {
            self.a.swap(1, i);
            self.heap_size -= 1;
            self.max_heapify(1);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DArrayHeap {
    a: Vec<u32>,
    d: usize,
    heap_size: usize,
}
impl DArrayHeap {
    pub fn build(a: &[u32], d: usize) -> Self {
        let mut me = Self {
            heap_size: a.len(),
            a: a.to_vec(),
            d,
        };
        me.build_max_heap();
        me
    }
    pub fn max_heapify(&mut self, i: usize) {
        let mut largest = i;
        for j in (self.d * i + 1..=self.d * (i + 1)).take_while(|&j| j < self.heap_size) {
            if self.a[largest] < self.a[j] {
                largest = j;
            }
        }
        if largest != i {
            self.a.swap(i, largest);
            self.max_heapify(largest);
        }
    }
    pub fn build_max_heap(&mut self) {
        self.heap_size = self.a.len();
        (0..(self.a.len() + self.d - 1) / self.d)
            .rev()
            .for_each(|i| self.max_heapify(i));
    }
    pub fn sort(&mut self) {
        self.build_max_heap();
        for i in (1..self.a.len()).rev() {
            self.heap_size -= 1;
            self.a.swap(0, i);
            self.max_heapify(0);
        }
    }
    pub fn increase_key(&mut self, mut i: usize, key: u32) {
        assert!(self.a[i] <= key);
        self.a[i] = key;
        while i != 0 && self.a[self.parent(i)] < self.a[i] {
            let p = self.parent(i);
            self.a.swap(i, p);
            i = p;
        }
    }
    pub fn insert(&mut self, key: u32) {
        self.heap_size += 1;
        if self.a.len() < self.heap_size {
            self.a.push(0);
        }
        self.a[self.heap_size - 1] = std::u32::MIN;
        self.increase_key(self.heap_size - 1, key);
    }
    fn parent(&self, i: usize) -> usize {
        (i - 1) / self.d
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryHeap, DArrayHeap};

    #[test]
    fn test_binary_heap() {
        let heap = vec![std::u32::MIN, 15, 13, 9, 5, 12, 8, 7, 4, 0, 6, 2, 1];
        let heap = BinaryHeap {
            heap_size: heap.len() - 1,
            a: heap,
        };
        {
            let mut heap = heap.clone();
            let result = heap.extract_max();
            assert_eq!(result, 15);
            let result = [std::u32::MIN, 13, 12, 9, 5, 6, 8, 7, 4, 0, 1, 2, 1];
            assert_eq!(&result, heap.a.as_slice());
        }
        {
            let mut heap = heap.clone();
            heap.insert(10);
            let result = [std::u32::MIN, 15, 13, 10, 5, 12, 9, 7, 4, 0, 6, 2, 1, 8];
            assert_eq!(&result, heap.a.as_slice());
        }
    }

    #[test]
    fn test_heap_sort() {
        let heap = vec![std::u32::MIN, 5, 13, 2, 25, 7, 17, 20, 8, 4];
        let mut heap = BinaryHeap {
            heap_size: heap.len() - 1,
            a: heap,
        };
        heap.sort();
        let result = [std::u32::MIN, 2, 4, 5, 7, 8, 13, 17, 20, 25];
        assert_eq!(&result, heap.a.as_slice());
    }

    #[test]
    fn test_tertiary_max_heap() {
        let heap = vec![5, 13, 2, 25, 7, 17, 20, 8, 4, 24, 3, 2, 14, 12, 9, 10, 1];
        let heap = DArrayHeap {
            heap_size: heap.len() - 1,
            a: heap,
            d: 3,
        };
        {
            let mut heap = heap.clone();
            heap.build_max_heap();
            let result = [25, 20, 24, 14, 12, 17, 13, 8, 4, 2, 3, 2, 5, 7, 9, 10, 1];
            assert_eq!(&result, heap.a.as_slice());

            heap.insert(24);
            let result = [
                25, 24, 24, 14, 12, 20, 13, 8, 4, 2, 3, 2, 5, 7, 9, 10, 1, 17,
            ];
            assert_eq!(&result, heap.a.as_slice());
        }
        {
            let mut heap = heap.clone();
            heap.sort();
            let result = [1, 2, 2, 3, 4, 5, 7, 8, 9, 10, 12, 13, 14, 17, 20, 24, 25];
            assert_eq!(&result, heap.a.as_slice());
        }
    }
}
