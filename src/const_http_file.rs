use bytedata::ByteData;

use crate::{HttpFile, HttpFileResponse};

/// A static HTTP file that can be computed at compile time or in other constant contexts.
///
/// The easiest way to create a `ConstHttpFile` is with the [`const_http_file!`] macro.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ConstHttpFile {
    pub file: Option<&'static str>,
    pub data: &'static [u8],
    pub mime: &'static str,
    pub etag: &'static str,
}

impl ConstHttpFile {
    /// Create a new [`ConstHttpFile`] with an explicit filename.
    pub const fn new_named(
        data: &'static [u8],
        mime: &'static str,
        etag: &'static str,
        file: &'static str,
    ) -> Self {
        ConstHttpFile {
            file: Some(file),
            data,
            mime,
            etag,
        }
    }

    /// Create a new [`ConstHttpFile`] without an explicit filename.
    pub const fn new(data: &'static [u8], mime: &'static str, etag: &'static str) -> Self {
        ConstHttpFile {
            file: None,
            data,
            mime,
            etag,
        }
    }

    pub const fn const_etag_str(&self) -> &'static str {
        if self.etag.is_empty() || !bytedata::const_starts_with(self.etag.as_bytes(), b"\"") {
            self.etag
        } else if let Some(a) = bytedata::const_slice_str(self.etag, 1..(self.etag.len() - 1)).ok() {
            a
        } else {
            panic!("Invalid etag in ConstHttpFile")
        }
    }
}

impl Default for ConstHttpFile {
    fn default() -> Self {
        ConstHttpFile {
            file: None,
            data: &[],
            mime: "application/octet-data",
            etag: "",
        }
    }
}

impl HttpFile<'static> for ConstHttpFile {
    fn content_type(&self) -> &str {
        self.mime
    }

    fn etag(&self) -> &str {
        self.etag
    }

    fn data(&self) -> &[u8] {
        self.data
    }

    fn into_data(self) -> ByteData<'static> {
        ByteData::from_static(self.data)
    }

    fn clone_data(&self) -> ByteData<'static> {
        ByteData::from_static(self.data)
    }
}

impl HttpFileResponse<'static> for ConstHttpFile {}

/// Create a [`ConstHttpFile`] from a file path or bytes. An explicit MIME type can also be provided.
///
/// If no MIME type is provided, it will be detected from the file extension or file contents.
///
/// # Examples
///
/// ```
/// # use static_http_file::{ConstHttpFile, const_http_file};
/// /// Explicit MIME type provided.
/// const FILE_0: ConstHttpFile = const_http_file!("../.gitignore", "text/plain; charset=utf-8");
///
/// /// No MIME type provided, so it will be detected from the file extension or file contents.
/// /// Unfortunately, `.gitignore` files are not in the detection list for file extensions and have no detectable early content,
/// /// so the MIME type will default to `application/octet-data`.
/// const FILE_1: ConstHttpFile = const_http_file!("../.gitignore");
///
/// const FILE_2_BYTES: &[u8] = include_bytes!("../.gitignore");
/// /// If the first argument is a non-literal expression, it will be used as the file contents instead of as a build-time path.
/// const FILE_2: ConstHttpFile = const_http_file!(FILE_2_BYTES, "text/plain; charset=utf-8");
/// ```
#[macro_export]
macro_rules! const_http_file {
    ($file:literal, $mime:expr) => {{
        const __FILE_BYTES: &[u8] = include_bytes!($file);
        const __FILE_ETAG: &str = $crate::const_etag!(__FILE_BYTES);
        $crate::ConstHttpFile::new_named(__FILE_BYTES, $mime, __FILE_ETAG, $file)
    }};
    ($file:literal) => {{
        const __FILE_BYTES: &[u8] = include_bytes!($file);
        const __FILE_ETAG: &str = $crate::const_etag!(__FILE_BYTES);
        const __FILE_MIME: &str = ::bytedata::const_or_str(
            $crate::detect_mime_type($file, __FILE_BYTES),
            "application/octet-data",
        );
        $crate::ConstHttpFile::new_named(__FILE_BYTES, __FILE_MIME, __FILE_ETAG, $file)
    }};
    ($file:expr, $mime:expr) => {{
        const __FILE_BYTES: &[u8] = $file;
        const __FILE_ETAG: &str = $crate::const_etag!(__FILE_BYTES);
        $crate::ConstHttpFile::new(__FILE_BYTES, $mime, __FILE_ETAG)
    }};
    ($file:expr) => {{
        const __FILE_BYTES: &[u8] = $file;
        const __FILE_ETAG: &str = $crate::const_etag!(__FILE_BYTES);
        const __FILE_MIME: &str = ::bytedata::const_or_str(
            $crate::detect_mime_type_magic(__FILE_BYTES),
            "application/octet-data",
        );
        $crate::ConstHttpFile::new(__FILE_BYTES, __FILE_MIME, __FILE_ETAG)
    }};
}
