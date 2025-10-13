use ecow::EcoVec;

/// Generic helper to process commits (or any indexed items) sequentially or in parallel.
/// Accepts any iterator over (index, item) and a processing function.
pub fn process_indexed<I, F, T, U>(iter: I, mut process: F) -> EcoVec<U>
where
    I: Iterator<Item = (usize, T)>,
    F: FnMut(usize, T) -> Option<U>,
    U: Clone,
{
    let mut out = EcoVec::new();
    for (idx, item) in iter {
        if let Some(result) = process(idx, item) {
            out.push(result);
        }
    }
    out
}
