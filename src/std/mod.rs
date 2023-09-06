mod std_http_file;
pub use std_http_file::*;

/// Compute an etag from a byte slice. The returned etag is a base64url-encoded 64-bit xxhash3 hash of the data wrapped in quotes.
///
/// Example:
/// ```
/// # use static_http_file::compute_etag_nonconst;
/// let etag: String = compute_etag_nonconst(b"foo");
/// assert_eq!(&etag, "\"q25fZAd-fY\"");
/// ```
pub fn compute_etag_nonconst(data: &[u8]) -> String {
    let h = xxhash_rust::xxh3::xxh3_64(data).to_be_bytes();
    let (mut etag, _n) = crate::b64url_const(&h, [0; 12], 1);
    #[cfg(debug_assertions)]
    if _n != 12 {
        panic!("Unexpected etag length");
    }
    etag[0] = b'"';
    etag[11] = b'"';
    unsafe { String::from_utf8_unchecked(etag.to_vec()) }
}
