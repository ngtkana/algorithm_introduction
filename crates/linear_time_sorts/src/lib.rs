pub fn counting_sort(a: &mut [u32], lim: u32) {
    let mut c = vec![0; lim as usize];
    a.iter().for_each(|&x| c[x as usize] += 1);
    a.copy_from_slice(
        &c.iter()
            .copied()
            .enumerate()
            .map(|(i, x)| std::iter::repeat(i as u32).take(x))
            .flatten()
            .collect::<Vec<_>>(),
    );
}

pub fn radix_sort(a: &mut [u32], r: usize) {
    let b = 32;
    for i in 0..(b + r - 1) / r {
        let mut c = vec![0; 1 << r];
        a.iter()
            .map(|&x| extract_key(x, i, r))
            .for_each(|key| c[key] += 1);
        for i in 1..1 << r {
            c[i] += c[i - 1];
        }

        let mut swp = vec![0; a.len()];
        for &x in a.iter().rev() {
            let key = extract_key(x, i, r);
            c[key] -= 1;
            swp[c[key]] = x;
        }
        a.copy_from_slice(&swp);
    }

    fn extract_key(x: u32, i: usize, r: usize) -> usize {
        ((x >> i * r) & (1 << r) - 1) as usize
    }
}

pub fn backet_sort(a: &mut [u32], lim: u32) {
    let n = a.len();
    let mut b = vec![Vec::new(); n];
    a.iter()
        .for_each(|&x| b[x as usize * n / lim as usize].push(x));
    b.iter_mut().for_each(|list| insertion_sort(list));
    a.copy_from_slice(&b.iter().flatten().copied().collect::<Vec<_>>());
}

pub fn insertion_sort(a: &mut [u32]) {
    for j in 1..a.len() {
        let key = a[j];
        for i in (0..=j).rev() {
            if i != j {
                a[i + 1] = a[i];
            }
            if i == 0 || a[i - 1] <= key {
                a[i] = key;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{backet_sort, counting_sort, insertion_sort, radix_sort};
    use rand::prelude::*;

    #[test]
    fn test_insertion_sort() {
        let mut a = [
            2, 8, 6, 9, 4, 7, 8, 0, 3, 5, 6, 1, 8, 6, 2, 9, 6, 0, 2, 7, 3, 0, 1,
        ];
        let expected = [
            0, 0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 6, 6, 6, 7, 7, 8, 8, 8, 9, 9,
        ];
        insertion_sort(&mut a);
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_counting_sort() {
        let mut a = [
            2, 8, 6, 9, 4, 7, 8, 0, 3, 5, 6, 1, 8, 6, 2, 9, 6, 0, 2, 7, 3, 0, 1,
        ];
        let expected = [
            0, 0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 5, 6, 6, 6, 6, 7, 7, 8, 8, 8, 9, 9,
        ];
        counting_sort(&mut a, 10);
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_radix_sort() {
        let n = 20;
        let mut rng = StdRng::seed_from_u64(32);
        let a = std::iter::repeat_with(|| rng.gen_range(0, std::u32::MAX))
            .take(n)
            .collect::<Vec<_>>();
        let expected = {
            let mut expected = a.clone();
            expected.sort();
            expected
        };
        let mut a = a;
        radix_sort(&mut a, 3);
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_backet_sort() {
        let n = 20;
        let lim = 1_000_000;
        let mut rng = StdRng::seed_from_u64(32);
        let a = std::iter::repeat_with(|| rng.gen_range(0, lim))
            .take(n)
            .collect::<Vec<_>>();
        let expected = {
            let mut expected = a.clone();
            expected.sort();
            expected
        };
        let mut a = a;
        backet_sort(&mut a, lim);
        assert_eq!(&a, &expected);
    }
}
