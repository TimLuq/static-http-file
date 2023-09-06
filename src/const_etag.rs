/// Compute an etag from a byte slice. The returned etag is a base64url-encoded `&str` from a 64-bit xxhash3 hash of the data wrapped in quotes.
///
/// Example:
/// ```
/// # use static_http_file::const_etag;
/// const ETAG: &str = const_etag!(b"foo");
/// assert_eq!(ETAG, "\"q25fZAd-fY\"");
/// ```
#[macro_export]
macro_rules! const_etag {
    ($data:expr) => {{
        const __FILE_ETAG: &[u8; 12] = &$crate::compute_etag($data);
        const __FILE_ETAG_STR: &str = unsafe { core::str::from_utf8_unchecked(__FILE_ETAG) };
        __FILE_ETAG_STR
    }};
}

/// Compute an etag from a byte slice. The returned etag is a base64url-encoded 64-bit xxhash3 hash of the data wrapped in quotes.
///
/// Example:
/// ```
/// # use static_http_file::compute_etag;
/// const ETAG: [u8; 12] = compute_etag(b"foo");
/// assert_eq!(&ETAG, b"\"q25fZAd-fY\"");
/// ```
pub const fn compute_etag(data: &[u8]) -> [u8; 12] {
    let h = xxhash_rust::const_xxh3::xxh3_64(data).to_be_bytes();
    let (mut etag, _n) = crate::b64url_const(&h, [0; 12], 1);
    #[cfg(debug_assertions)]
    if _n != 12 {
        panic!("Unexpected etag length");
    }
    etag[0] = b'"';
    etag[11] = b'"';
    etag
}
