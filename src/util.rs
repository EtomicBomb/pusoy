pub fn windows2<T>(slice: &[T]) -> impl Iterator<Item=(&T, &T)> {
    // like a slice.windows(2) call, but can never produce []

    assert!(slice.len() >= 2);

    (0..slice.len()-1)
        .map(move |i| (&slice[i], &slice[i+1]))
}

#[inline]
pub fn counter(base: &[usize], mut f: impl FnMut(&[usize])) {
    // a generalized version of counting in an arbitrary base
    // calls f on each number generated in the count
    // for example, counter(&[2; 3], f) calls f on:
    //      &[0, 0, 0]
    //      &[1, 0, 0]
    //      &[0, 1, 0]
    //      &[1, 1, 0]
    //      etc.

    let len = base.len();

    let mut x = vec![0; len];

    let iter_count: usize = base.iter().product();

    for _ in 0..iter_count {
        f(&x);

        // try to "add one"
        for i in 0..len {
            if x[i] < base[i] - 1 {
                x[i] += 1;
                break;
            }

            x[i] = 0;
        }
    }
}

#[inline]
pub fn concat<T>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<T> {
    a.append(&mut b);
    a
}