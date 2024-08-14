pub fn get_pair_mut<T>(slice: &mut [T], a: usize, b: usize) -> Option<(&mut T, &mut T)> {
    let (first, second) = (usize::min(a, b), usize::max(a, b));

    let [first, .., second] = slice.get_mut(first..=second)? else {
        return None;
    };

    let pair = if a < b {
        (first, second)
    } else {
        (second, first)
    };
    Some(pair)
}
