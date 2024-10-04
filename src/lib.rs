#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod const_mime;
use core::num::NonZeroU8;

use bytedata::StringData;
pub use const_mime::*;
mod traits;
pub use traits::*;

mod const_http_file;
pub use const_http_file::ConstHttpFile;

mod cachebusted_http_file;
pub use cachebusted_http_file::QueryCacheBustedHttpFile;

mod const_etag;
pub use const_etag::*;

mod const_b64;
pub use const_b64::*;

#[cfg(feature = "std")]
mod std;
#[cfg(feature = "std")]
pub use self::std::*;

#[cfg(feature = "tokio_1")]
mod tokio_1;
#[cfg(feature = "tokio_1")]
pub use self::tokio_1::*;

#[cfg(feature = "expose")]
mod expose;
#[cfg(feature = "expose")]
pub use self::expose::*;

#[cfg(feature = "http_1")]
pub mod http_1;

#[cfg(feature = "http_02")]
pub mod http_02;

#[cfg(feature = "http_02_reexport")]
pub use http_02::*;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum CacheBusting {
    #[default]
    None,
    /// Cachebust by using the etag in the query string.
    /// If used as `Query("foo")`, the query string will be something like `?foo=q25fZAd-fY`.
    Query(StringData<'static>),
    /// Cachebust by using the etag in the filename.
    /// The first byte of the suffix is the separator between the basename and the etag.
    /// The request path is expected to always contain an etag.
    Suffix(Option<NonZeroU8>),
}

#[cfg(test)]
mod test;
