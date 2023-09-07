use core::num::NonZeroU8;
use std::path::Path;

use alloc::borrow::Cow;
use bytedata::ByteData;

use super::super::std::{compute_etag_nonconst, StdHttpFile};
use crate::{HttpFile, HttpFileResponse};

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
    #[inline]
    fn content_type(&self) -> &str {
        self.inner.mime.as_ref()
    }

    #[inline]
    fn etag(&self) -> &str {
        self.inner.etag.as_ref()
    }

    #[inline]
    fn data(&self) -> &[u8] {
        self.inner.data.as_slice()
    }

    #[inline]
    fn into_data(self) -> ByteData<'static> {
        self.inner.into_data()
    }

    #[inline]
    fn clone_data(&self) -> ByteData<'static> {
        self.inner.data.clone()
    }
}

impl HttpFileResponse<'static> for TokioHttpFile {
    #[inline]
    fn respond_guard<T: From<ByteData<'static>>>(
        &self,
        request: &http::Request<()>,
    ) -> Result<http::response::Builder, Result<http::Response<T>, http::Error>> {
        self.inner.respond_guard(request)
    }

    #[inline]
    fn respond<T: From<ByteData<'static>>>(
        self,
        request: &http::Request<()>,
    ) -> Result<http::Response<T>, http::Error> {
        self.inner.respond(request)
    }

    #[inline]
    fn respond_borrowed<T: From<ByteData<'static>>>(
        &self,
        request: &http::Request<()>,
    ) -> Result<http::Response<T>, http::Error> {
        self.inner.respond_borrowed(request)
    }

    #[inline]
    fn response_headers(&self, response: http::response::Builder) -> http::response::Builder {
        self.inner.response_headers(response)
    }

    #[inline]
    fn into_response<T: From<ByteData<'static>>>(self) -> Result<http::Response<T>, http::Error> {
        self.inner.into_response()
    }

    #[inline]
    fn cachebust_uri<T: From<ByteData<'static>>>(
        &self,
        old_uri: &http::Uri,
        query_key: &str,
    ) -> Option<Result<http::Response<T>, http::Error>> {
        self.inner.cachebust_uri(old_uri, query_key)
    }

    #[inline]
    fn cachebust_suffix<T: From<ByteData<'static>>>(
        &self,
        old_uri: &http::Uri,
        left_sep: Option<NonZeroU8>,
    ) -> Option<Result<http::Response<T>, http::Error>> {
        self.inner.cachebust_suffix(old_uri, left_sep)
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
