use std::{fs::File, path::Path};

use alloc::borrow::Cow;
use bytedata::ByteData;

use crate::HttpFile;

/// A static HTTP file that can be computed at compile time or in other constant contexts.
///
/// The easiest way to create a `StdHttpFile` is with the [`const_http_file!`] macro.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StdHttpFile {
    pub file: Cow<'static, str>,
    pub data: ByteData<'static>,
    pub mime: Cow<'static, str>,
    pub etag: Cow<'static, str>,
}

impl StdHttpFile {
    /// Create a new [`StdHttpFile`] with an explicit mime, data, and etag.
    pub const fn new_with_mime_data_etag(
        file: Cow<'static, str>,
        mime: Cow<'static, str>,
        data: ByteData<'static>,
        etag: Cow<'static, str>,
    ) -> Self {
        StdHttpFile {
            file,
            data,
            mime,
            etag,
        }
    }

    /// Create a new [`StdHttpFile`] with an explicit mime and data.
    pub fn new_with_mime_data(
        file: Cow<'static, str>,
        mime: Cow<'static, str>,
        data: ByteData<'static>,
    ) -> Self {
        let etag = super::compute_etag_nonconst(data.as_slice());
        StdHttpFile {
            file,
            data,
            mime,
            etag: Cow::Owned(etag),
        }
    }

    /// Create a new [`StdHttpFile`] from a path.
    pub fn new(path: impl Into<Cow<'static, str>>) -> std::io::Result<Self> {
        let path: Cow<'static, str> = path.into();
        let data = read_file(path.as_ref().as_ref())?;
        let mime =
            crate::detect_mime_type(path.as_ref(), &data).unwrap_or("application/octet-data");
        let etag = super::compute_etag_nonconst(&data);
        Ok(StdHttpFile {
            file: path,
            data: ByteData::from_shared(data),
            mime: Cow::Borrowed(mime),
            etag: Cow::Owned(etag),
        })
    }

    /// Create a new [`StdHttpFile`] from a file and explicit mime.
    pub fn new_with_mime(
        path: impl Into<Cow<'static, str>>,
        mime: impl Into<Cow<'static, str>>,
    ) -> std::io::Result<Self> {
        let path: Cow<'static, str> = path.into();
        let data = read_file(path.as_ref().as_ref())?;
        let etag = super::compute_etag_nonconst(&data);
        Ok(StdHttpFile {
            file: path,
            data: ByteData::from_shared(data),
            mime: mime.into(),
            etag: Cow::Owned(etag),
        })
    }
}

impl HttpFile<'static> for StdHttpFile {
    fn content_type(&self) -> &str {
        self.mime.as_ref()
    }

    fn etag(&self) -> &str {
        self.etag.as_ref()
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn into_data(self) -> ByteData<'static> {
        self.data
    }

    fn clone_data(&self) -> ByteData<'static> {
        self.data.clone()
    }
}

fn read_file(path: &Path) -> std::io::Result<bytedata::SharedBytes> {
    let mut builder = bytedata::SharedBytesBuilder::new();
    read_file_into(path, &mut builder)?;
    Ok(builder.build())
}

fn read_file_into(path: &Path, builder: &mut bytedata::SharedBytesBuilder) -> std::io::Result<()> {
    use bytes_1::BufMut;
    use std::io::Read;
    let mut file = File::open(path)?;
    loop {
        let buf = builder.chunk_mut();
        let n =
            file.read(unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len()) })?;
        if n == 0 {
            break;
        }
        unsafe { builder.advance_mut(n) };
    }
    Ok(())
}
