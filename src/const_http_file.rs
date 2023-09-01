use crate::HttpFile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ConstHttpFile {
    pub file: Option<&'static str>,
    pub data: &'static [u8],
    pub mime: &'static str,
    pub etag: &'static str,
}

impl ConstHttpFile {
    pub const fn new_named(data: &'static [u8], mime: &'static str, etag: &'static str, file: &'static str) -> Self {
        ConstHttpFile {
            file: Some(file),
            data,
            mime,
            etag,
        }
    }

    pub const fn new(data: &'static [u8], mime: &'static str, etag: &'static str) -> Self {
        ConstHttpFile {
            file: None,
            data,
            mime,
            etag,
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

    fn into_data(self) -> crate::FileData<'static> {
        crate::FileData::Static(self.data)
    }
}

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
        const __FILE_MIME: &str = $crate::const_or_str($crate::detect_mime_type($file, __FILE_BYTES), "application/octet-data");
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
        const __FILE_MIME: &str = $crate::const_or_str($crate::detect_mime_type_magic(__FILE_BYTES), "application/octet-data");
        $crate::ConstHttpFile::new(__FILE_BYTES, __FILE_MIME, __FILE_ETAG)
    }};
}
