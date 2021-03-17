use std::convert::TryInto;
use std::fmt::Debug;

/// Losslessly converts `n` to `usize` using `TryFrom` or panics.
///
/// # Panics
///
/// Panics if the conversion errors.
pub fn cast_usize<T>(n: T) -> usize
where
    T: TryInto<usize>,
    <T as TryInto<usize>>::Error: Debug,
{
    n.try_into().expect("Expected N to fit in usize")
}
