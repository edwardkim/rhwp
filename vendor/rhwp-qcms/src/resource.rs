//! Allocation and work of the originating import request.
pub(crate) fn capacity<T>(n: usize) -> Option<Vec<T>> {
    rhwp_codec_resource::work(n).ok()?;
    rhwp_codec_resource::capacity(n).ok()
}
pub(crate) fn copy<T: Copy>(source: &[T]) -> Option<Vec<T>> {
    rhwp_codec_resource::copy(source).ok()
}
pub(crate) fn work(n: usize) -> Option<()> {
    rhwp_codec_resource::work(n).ok()
}
