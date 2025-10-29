use bytedata::{ByteData, StringData};

use crate::HttpFile;

/// A HTTP file that can possibly be computed const at compile time, but may also be created runtime using owned or borrowed data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct SharedHttpFile<'a> {
    pub file: Option<StringData<'a>>,
    pub data: ByteData<'a>,
    pub mime: StringData<'a>,
    pub etag: StringData<'a>,
}

impl<'a> SharedHttpFile<'a> {
    /// Create a new [`SharedHttpFile`] with an explicit filename.
    pub const fn new_named(
        data: ByteData<'a>,
        mime: StringData<'a>,
        etag: StringData<'a>,
        file: StringData<'a>,
    ) -> Self {
        SharedHttpFile {
            file: Some(file),
            data,
            mime,
            etag,
        }
    }

    /// Create a new [`SharedHttpFile`] without an explicit filename.
    pub const fn new(data: ByteData<'a>, mime: StringData<'a>, etag: StringData<'a>) -> Self {
        SharedHttpFile {
            file: None,
            data,
            mime,
            etag,
        }
    }

    pub const fn const_etag_str(&self) -> &str {
        if self.etag.is_empty() || !bytedata::const_starts_with(self.etag.as_bytes(), b"\"") {
            self.etag.as_str()
        } else if let Some(a) =
            bytedata::const_slice_str(self.etag.as_str(), 1..(self.etag.len() - 1)).ok()
        {
            a
        } else {
            panic!("Invalid etag in SharedHttpFile")
        }
    }
}

impl Default for SharedHttpFile<'_> {
    fn default() -> Self {
        SharedHttpFile {
            file: None,
            data: ByteData::empty(),
            mime: StringData::from_static("application/octet-data"),
            etag: StringData::empty(),
        }
    }
}

impl<'a> HttpFile<'a> for SharedHttpFile<'a> {
    fn content_type(&self) -> &str {
        self.mime.as_str()
    }

    fn etag(&self) -> &str {
        self.etag.as_str()
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn into_data(self) -> ByteData<'a> {
        self.data
    }

    fn clone_data(&self) -> ByteData<'a> {
        self.data.clone()
    }
}

#[cfg(feature = "http_02")]
impl<'a> crate::http_02::HttpFileResponse<'a> for SharedHttpFile<'a> {}
#[cfg(feature = "http_1")]
impl<'a> crate::http_1::HttpFileResponse<'a> for SharedHttpFile<'a> {}
