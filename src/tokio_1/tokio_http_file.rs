use std::path::Path;

use alloc::borrow::Cow;
use bytedata::ByteData;

use super::super::std::{compute_etag_nonconst, StdHttpFile};
use crate::HttpFile;

/// A static HTTP file that can be computed at compile time or in other constant contexts.
///
/// The easiest way to create a `TokioHttpFile` is with the [`const_http_file!`] macro.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct TokioHttpFile {
    inner: StdHttpFile,
}

impl StdHttpFile {
    pub(crate) const fn into_tokio_file(self) -> TokioHttpFile {
        TokioHttpFile { inner: self }
    }
}

impl TokioHttpFile {
    /// Create a new [`TokioHttpFile`] with an explicit mime, data, and etag.
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
        .into_tokio_file()
    }

    /// Create a new [`TokioHttpFile`] with an explicit mime and data.
    pub fn new_with_mime_data(
        file: Cow<'static, str>,
        mime: Cow<'static, str>,
        data: ByteData<'static>,
    ) -> Self {
        let etag = compute_etag_nonconst(data.as_slice());
        StdHttpFile {
            file,
            data,
            mime,
            etag: Cow::Owned(etag),
        }
        .into_tokio_file()
    }

    /// Create a new [`TokioHttpFile`] from a path.
    pub async fn new(path: impl Into<Cow<'static, str>>) -> std::io::Result<Self> {
        let path: Cow<'static, str> = path.into();
        let data = read_file(path.as_ref().as_ref()).await?;
        let mime =
            crate::detect_mime_type(path.as_ref(), &data).unwrap_or("application/octet-data");
        let etag = compute_etag_nonconst(&data);
        Ok(StdHttpFile {
            file: path,
            data: ByteData::from_shared(data),
            mime: Cow::Borrowed(mime),
            etag: Cow::Owned(etag),
        }
        .into_tokio_file())
    }

    /// Create a new [`TokioHttpFile`] from a file and explicit mime.
    pub async fn new_with_mime(
        path: impl Into<Cow<'static, str>>,
        mime: impl Into<Cow<'static, str>>,
    ) -> std::io::Result<Self> {
        let path: Cow<'static, str> = path.into();
        let data = read_file(path.as_ref().as_ref()).await?;
        let etag = compute_etag_nonconst(&data);
        Ok(StdHttpFile {
            file: path,
            data: ByteData::from_shared(data),
            mime: mime.into(),
            etag: Cow::Owned(etag),
        }
        .into_tokio_file())
    }

    /// Transforms the result of a `TokioHttpFile` as a [`StdHttpFile`].
    pub const fn into_std_file(self) -> StdHttpFile {
        unsafe { core::mem::transmute::<TokioHttpFile, StdHttpFile>(self) }
    }
}

impl HttpFile<'static> for TokioHttpFile {
    fn content_type(&self) -> &str {
        self.inner.mime.as_ref()
    }

    fn etag(&self) -> &str {
        self.inner.etag.as_ref()
    }

    fn data(&self) -> &[u8] {
        self.inner.data.as_slice()
    }

    fn into_data(self) -> ByteData<'static> {
        self.inner.into_data()
    }

    fn clone_data(&self) -> ByteData<'static> {
        self.inner.data.clone()
    }
}

async fn read_file(path: &Path) -> std::io::Result<bytedata::SharedBytes> {
    let mut builder = bytedata::SharedBytesBuilder::new();
    read_file_into(path, &mut builder).await?;
    Ok(builder.build())
}

async fn read_file_into(
    path: &Path,
    builder: &mut bytedata::SharedBytesBuilder,
) -> std::io::Result<()> {
    use ::tokio_1::{fs::File, io::AsyncReadExt};
    use bytes_1::BufMut;
    let mut file = File::open(path).await?;
    loop {
        let buf = builder.chunk_mut();
        let n = file
            .read(unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len()) })
            .await?;
        if n == 0 {
            break;
        }
        unsafe { builder.advance_mut(n) };
    }
    Ok(())
}
