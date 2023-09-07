use core::num::NonZeroU8;

use alloc::format;
use bytedata::{ByteData, StringData};

use crate::{CacheBusting, HttpFile, HttpFileResponse};

pub struct QueryCacheBustedHttpFile<T> {
    url: StringData<'static>,
    cbust: CacheBusting,
    inner: T,
}

impl<'l, T: HttpFileResponse<'l>> QueryCacheBustedHttpFile<T> {
    /// Create a new [`QueryCacheBustedHttpFile`] from a precomputed URL.
    /// The URL must contain a query parameter matching the `query_var` parameter with the exact unquoted etag as value.
    pub const fn new_const(
        url: StringData<'static>,
        query_var: StringData<'static>,
        inner: T,
    ) -> Self {
        QueryCacheBustedHttpFile {
            url,
            cbust: CacheBusting::Query(query_var),
            inner,
        }
    }

    /// Create a new [`QueryCacheBustedHttpFile`] from a base URL and the name of the query parameter that will be used.
    pub fn new(
        file_url: impl AsRef<str>,
        query_var: impl Into<StringData<'static>>,
        inner: T,
    ) -> Self {
        let query_var = query_var.into();
        let url = format!(
            "{}?{}={}",
            file_url.as_ref(),
            query_var.as_str(),
            inner.etag()
        );
        QueryCacheBustedHttpFile {
            url: url.into(),
            cbust: CacheBusting::Query(query_var),
            inner,
        }
    }

    /// The cachebusted URL.
    pub const fn url(&self) -> &StringData<'static> {
        &self.url
    }
}

impl<'l, T: HttpFileResponse<'l>> HttpFile<'l> for QueryCacheBustedHttpFile<T> {
    #[inline]
    fn content_type(&self) -> &str {
        self.inner.content_type()
    }

    #[inline]
    fn etag(&self) -> &str {
        self.inner.etag()
    }

    #[inline]
    fn etag_str(&self) -> &str {
        self.inner.etag_str()
    }

    #[inline]
    fn cache_busting(&self) -> &CacheBusting {
        &self.cbust
    }

    #[inline]
    fn data(&self) -> &[u8] {
        self.inner.data()
    }

    #[inline]
    fn into_data(self) -> ByteData<'l> {
        self.inner.into_data()
    }

    #[inline]
    fn clone_data(&self) -> ByteData<'l> {
        self.inner.clone_data()
    }
}

impl<'l, T: HttpFileResponse<'l>> HttpFileResponse<'l> for QueryCacheBustedHttpFile<T> {
    #[inline]
    fn respond_guard<R: From<ByteData<'l>>>(
        &self,
        request: &http::Request<()>,
    ) -> Result<http::response::Builder, Result<http::Response<R>, http::Error>> {
        self.inner.respond_guard(request)
    }

    #[inline]
    fn respond<R: From<ByteData<'l>>>(
        self,
        request: &http::Request<()>,
    ) -> Result<http::Response<R>, http::Error> {
        self.inner.respond(request)
    }

    #[inline]
    fn respond_borrowed<R: From<ByteData<'l>>>(
        &self,
        request: &http::Request<()>,
    ) -> Result<http::Response<R>, http::Error> {
        self.inner.respond_borrowed(request)
    }

    #[inline]
    fn response_headers(&self, response: http::response::Builder) -> http::response::Builder {
        self.inner.response_headers(response)
    }

    #[inline]
    fn into_response<R: From<ByteData<'l>>>(self) -> Result<http::Response<R>, http::Error> {
        self.inner.into_response()
    }

    #[inline]
    fn cachebust_uri<R: From<ByteData<'l>>>(
        &self,
        old_uri: &http::Uri,
        query_key: &str,
    ) -> Option<Result<http::Response<R>, http::Error>> {
        self.inner.cachebust_uri(old_uri, query_key)
    }

    #[inline]
    fn cachebust_suffix<R: From<ByteData<'l>>>(
        &self,
        old_uri: &http::Uri,
        left_sep: Option<NonZeroU8>,
    ) -> Option<Result<http::Response<R>, http::Error>> {
        self.inner.cachebust_suffix(old_uri, left_sep)
    }
}


/// Create a [`ConstHttpFile`] from a file path. An explicit MIME type can also be provided.
///
/// If no MIME type is provided, it will be detected from the file extension or file contents.
///
/// # Examples
///
/// ```
/// # use static_http_file::{ConstHttpFile, QueryCacheBustedHttpFile, static_http_file_querycache};
/// /// Explicit MIME type provided.
/// static FILE_0: QueryCacheBustedHttpFile<ConstHttpFile> = static_http_file_querycache!("v_et", "../.gitignore", "text/plain; charset=utf-8");
/// assert_eq!(FILE_0.url().as_str(), "../.gitignore?v_et=bk4EOvJYzH");
/// static FILE_1: QueryCacheBustedHttpFile<ConstHttpFile> = static_http_file_querycache!("../.gitignore");
/// assert_eq!(FILE_1.url().as_str(), "../.gitignore?-=bk4EOvJYzH");
/// ```
#[macro_export]
macro_rules! static_http_file_querycache {
    ($queryvar:literal, $file:literal, $($r:tt)*) => {{
        const __FILE_CONST: $crate::ConstHttpFile = $crate::const_http_file!($file, $($r)*);
        const __FILE_ETAG: &str = __FILE_CONST.const_etag_str();
        static __FILE_URL: &str = ::bytedata::concat_str_static!($file, "?", $queryvar, "=", __FILE_ETAG);
        const __FILE_QVAR: bytedata::StringData = bytedata::StringData::from_static($queryvar);
        $crate::QueryCacheBustedHttpFile::new_const(bytedata::StringData::from_static(__FILE_URL), __FILE_QVAR, __FILE_CONST)
    }};
    ($queryvar:literal, $file:literal) => {{
        const __FILE_CONST: $crate::ConstHttpFile = $crate::const_http_file!($file);
        const __FILE_ETAG: &str = __FILE_CONST.const_etag_str();
        static __FILE_URL: &str = ::bytedata::concat_str_static!($file, "?", $queryvar, "=", __FILE_ETAG);
        const __FILE_QVAR: bytedata::StringData = bytedata::StringData::from_static($queryvar);
        $crate::QueryCacheBustedHttpFile::new_const(bytedata::StringData::from_static(__FILE_URL), __FILE_QVAR, __FILE_CONST)
    }};
    ($file:literal) => {{
        const __FILE_CONST: $crate::ConstHttpFile = $crate::const_http_file!($file);
        const __FILE_ETAG: &str = __FILE_CONST.const_etag_str();
        static __FILE_URL: &str = ::bytedata::concat_str_static!($file, "?-=", __FILE_ETAG);
        const __FILE_QVAR: bytedata::StringData = bytedata::StringData::from_static("-");
        $crate::QueryCacheBustedHttpFile::new_const(bytedata::StringData::from_static(__FILE_URL), __FILE_QVAR, __FILE_CONST)
    }};
}
