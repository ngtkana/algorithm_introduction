pub fn quick_sort(a: &mut [u32]) {
    if 2 <= a.len() {
        let i = partition(a);
        quick_sort(&mut a[..i]);
        quick_sort(&mut a[i + 1..]);
    }
}

pub fn randomized_quick_sort(a: &mut [u32]) {
    use rand::prelude::*;
    let mut rng = StdRng::seed_from_u64(42);
    pub fn dfs(a: &mut [u32], rng: &mut StdRng) {
        let n = a.len();
        if 2 <= n {
            let i = rng.gen_range(0, n);
            a.swap(i, n - 1);
            let i = partition(a);
            dfs(&mut a[..i], rng);
            dfs(&mut a[i + 1..], rng);
        }
    }
    dfs(a, &mut rng);
}

fn partition(a: &mut [u32]) -> usize {
    let n = a.len();
    assert!(2 <= n);
    let x = a[n - 1];
    let mut i = 0;
    for j in 0..n - 1 {
        if a[j] < x {
            a.swap(i, j);
            i += 1;
        }
    }
    a.swap(i, n - 1);
    i
}

#[cfg(test)]
mod tests {
    use super::{quick_sort, randomized_quick_sort};

    #[test]
    fn test_quick_sort() {
        let mut a = [
            3, 4, 6, 8, 5, 9, 1, 2, 4, 2, 2, 3, 4, 5, 6, 3, 7, 8, 4, 3, 2, 5, 6, 6, 6, 3,
        ];
        quick_sort(&mut a);
        let expected = [
            1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 6, 6, 6, 6, 6, 7, 8, 8, 9,
        ];
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_randomized_quick_sort() {
        let mut a = [
            3, 4, 6, 8, 5, 9, 1, 2, 4, 2, 2, 3, 4, 5, 6, 3, 7, 8, 4, 3, 2, 5, 6, 6, 6, 3,
        ];
        randomized_quick_sort(&mut a);
        let expected = [
            1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 6, 6, 6, 6, 6, 7, 8, 8, 9,
        ];
        assert_eq!(&a, &expected);
    }
}
