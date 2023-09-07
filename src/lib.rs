#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod const_mime;
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

#[cfg(test)]
mod test;
