// code was taken from `permutation` crate:
// https://github.com/jeremysalwen/rust-permutations/blob/5528e4fec7c5eb4551cfb39029c8d7982be2e6a4/src/permutation.rs#L400
// dependency was not used because he lack of `#[no_std]` attribute
#[inline]
pub fn apply<T>(permutation: &mut [usize], data: &mut [T]) {
    assert_eq!(permutation.len(), data.len());
    assert!(data.len() <= isize::MAX as usize);

    const MARKER: usize = isize::MIN as usize;

    #[inline(always)]
    fn idx_toggle_mark(idx: usize) -> usize {
        idx ^ MARKER
    }

    #[inline(always)]
    fn idx_is_marked(idx: usize) -> bool {
        (idx & MARKER) != 0
    }

    for idx in permutation.iter() {
        debug_assert!(!idx_is_marked(*idx));
    }

    for i in 0..permutation.len() {
        let i_idx = permutation[i];
        if idx_is_marked(i_idx) {
            continue;
        }

        let mut j = i;
        let mut j_idx = i_idx;
        while j_idx != i {
            permutation[j] = idx_toggle_mark(j_idx);
            data.swap(j, j_idx);
            j = j_idx;
            j_idx = permutation[j];
        }
        permutation[j] = idx_toggle_mark(j_idx);
    }

    for idx in permutation.iter_mut() {
        debug_assert!(idx_is_marked(*idx));
        *idx = idx_toggle_mark(*idx);
    }
}
