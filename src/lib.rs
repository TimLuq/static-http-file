#![no_std]

extern crate alloc;

mod const_mime;
pub use const_mime::*;
mod traits;
pub use traits::*;

mod const_http_file;
pub use const_http_file::ConstHttpFile;

mod const_etag;
pub use const_etag::*;

mod const_b64;
pub use const_b64::*;

pub const fn const_or_str<'a>(value: Option<&'a str>, default: &'a str) -> &'a str {
    match value {
        Some(value) => value,
        None => default,
    }
}

#[cfg(test)]
mod test;
