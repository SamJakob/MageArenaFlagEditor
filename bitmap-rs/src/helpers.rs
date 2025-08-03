use crate::Error;
use crate::Error::IllegalParameter;

/// Convert the given slice into an array of length `L`.
pub(crate) fn array_from_slice<T, const L: usize>(slice: &[T]) -> Result<&[T; L], Error> {
    slice.try_into().map_err(|_| IllegalParameter("slice length mismatch"))
}
