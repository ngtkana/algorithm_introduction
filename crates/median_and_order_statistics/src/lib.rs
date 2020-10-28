use rand::prelude::*;
use std::cmp::Ordering;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::seed_from_u64(42));
}

pub fn median_of_median_select(a: &mut [u32], k: usize) -> u32 {
    assert!(k < a.len());
    let mid = median_of_medians_partition(a);
    match k.cmp(&mid) {
        Ordering::Equal => a[k],
        Ordering::Less => randomized_select(&mut a[..mid], k),
        Ordering::Greater => randomized_select(&mut a[mid + 1..], k - mid - 1),
    }
}

pub fn median_of_medians_partition(a: &mut [u32]) -> usize {
    let mut b = a
        .chunks_mut(5)
        .map(|v| {
            v.sort();
            v[(v.len() - 1) / 2]
        })
        .collect::<Vec<_>>();
    let m = b.len();
    let median = median_of_median_select(&mut b, (m - 1) / 2);
    a.iter().position(|&x| x == median).unwrap()
}

pub fn randomized_select(a: &mut [u32], k: usize) -> u32 {
    assert!(k < a.len());
    let mid = randomized_partition(a);
    match k.cmp(&mid) {
        Ordering::Equal => a[k],
        Ordering::Less => randomized_select(&mut a[..mid], k),
        Ordering::Greater => randomized_select(&mut a[mid + 1..], k - mid - 1),
    }
}

pub fn randomized_select_no_recursion(a: &mut [u32], k: usize) -> u32 {
    assert!(k < a.len());
    let mut l = 0;
    let mut r = a.len();
    while l + 1 != r {
        let mid = l + randomized_partition(&mut a[l..r]);
        match k.cmp(&mid) {
            Ordering::Equal => {
                return a[k];
            }
            Ordering::Less => r = mid,
            Ordering::Greater => l = mid + 1,
        }
    }
    a[l]
}

pub fn randomized_partition(a: &mut [u32]) -> usize {
    let n = a.len();
    if n == 1 {
        0
    } else {
        a.swap(RNG.lock().unwrap().gen_range(0, n - 1), n - 1);
        partition(a)
    }
}

pub fn partition(a: &mut [u32]) -> usize {
    let n = a.len();
    if n == 1 {
        0
    } else {
        let key = a[n - 1];
        let mut i = 0;
        for j in 0..n - 1 {
            if a[j] < key {
                a.swap(i, j);
                i += 1;
            }
        }
        a.swap(i, n - 1);
        i
    }
}

#[cfg(test)]
mod tests {
    use super::{randomized_select, randomized_select_no_recursion};

    fn tester(select: impl Fn(&mut [u32], usize) -> u32) {
        let a = vec![3, 2, 9, 0, 7, 5, 4, 8, 6, 1];
        let mut sorted = a.clone();
        sorted.sort();
        let sorted = sorted;

        for i in 0..a.len() {
            let mut a = a.clone();
            let result = select(&mut a, i);
            assert_eq!(result, sorted[i]);
        }
    }

    #[test]
    fn test_randomized_select() {
        tester(randomized_select);
    }

    #[test]
    fn test_randomized_select_no_recursion() {
        tester(randomized_select_no_recursion);
    }

    #[test]
    fn test_median_of_medians_select() {
        tester(randomized_select_no_recursion);
    }
}
