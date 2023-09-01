use core::num::NonZeroU8;

use alloc::{boxed::Box, string::String, vec::Vec};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum CacheBusting {
    #[default]
    None,
    Query(&'static str),
    /// Cachebust by using the etag in the filename.
    /// The first byte of the suffix is the separator between the basename and the etag.
    /// The request path is expected to always contain an etag.
    Suffix(Option<NonZeroU8>),
}

pub trait HttpFile<'a> {
    /// Returns the content type of the file.
    fn content_type(&self) -> &str;
    /// Returns the data of the file.
    fn data(&self) -> &[u8];
    /// Returns the etag of the file (including quotes).
    fn etag(&self) -> &str;
    /// Returns the etag without quotes.
    fn etag_str(&self) -> &str {
        let e = self.etag();
        if e.len() > 2 && e.starts_with('"') && e.ends_with('"') {
            &e[1..e.len() - 1]
        } else {
            e
        }
    }
    /// Returns the cache busting method.
    fn cache_busting(&self) -> CacheBusting {
        CacheBusting::None
    }
    /// Extracts the data of the file.
    fn into_data(self) -> FileData<'a>;
    /// Clones the data of the file. This may only copy the reference if the data is not owned.
    fn clone_data(&self) -> FileData<'a>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileData<'a> {
    Static(&'static [u8]),
    Borrowed(&'a [u8]),
    Owned(Box<[u8]>),
}

impl<'a> FileData<'a> {
    /// Returns the data as a slice of bytes.
    pub const fn as_slice(&self) -> &[u8] {
        match self {
            FileData::Static(data) => data,
            FileData::Borrowed(data) => data,
            FileData::Owned(data) => data,
        }
    }

    /// Creates a new `FileData` from a static byte slice.
    pub const fn from_static(data: &'static [u8]) -> Self {
        FileData::Static(data)
    }

    /// Creates a new `FileData` from a byte slice.
    pub const fn from_slice(data: &'a [u8]) -> Self {
        FileData::Borrowed(data)
    }

    /// Creates a new `FileData` from a `Vec<u8>`.
    pub fn from_vec(data: Vec<u8>) -> Self {
        FileData::Owned(data.into_boxed_slice())
    }

    /// Creates a new `FileData` from a `Vec<u8>`.
    pub const fn from_boxed(data: Box<[u8]>) -> Self {
        FileData::Owned(data)
    }

    /// Returns `true` if the data contains no bytes.
    pub const fn is_empty(&self) -> bool {
        match self {
            FileData::Static(data) => data.is_empty(),
            FileData::Borrowed(data) => data.is_empty(),
            FileData::Owned(data) => data.is_empty(),
        }
    }

    /// Returns the length of the data in bytes.
    pub const fn len(&self) -> usize {
        match self {
            FileData::Static(data) => data.len(),
            FileData::Borrowed(data) => data.len(),
            FileData::Owned(data) => data.len(),
        }
    }
}

impl AsRef<[u8]> for FileData<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> From<&'a [u8]> for FileData<'a> {
    fn from(data: &'a [u8]) -> Self {
        FileData::Borrowed(data)
    }
}

impl From<Vec<u8>> for FileData<'_> {
    fn from(data: Vec<u8>) -> Self {
        FileData::Owned(data.into_boxed_slice())
    }
}

impl From<Box<[u8]>> for FileData<'_> {
    fn from(data: Box<[u8]>) -> Self {
        FileData::Owned(data)
    }
}

impl From<FileData<'_>> for Vec<u8> {
    fn from(data: FileData<'_>) -> Self {
        match data {
            FileData::Static(data) => data.to_vec(),
            FileData::Borrowed(data) => data.to_vec(),
            FileData::Owned(data) => data.into(),
        }
    }
}

impl From<FileData<'_>> for Box<[u8]> {
    fn from(data: FileData<'_>) -> Self {
        match data {
            FileData::Static(data) => Box::from(data),
            FileData::Borrowed(data) => Box::from(data),
            FileData::Owned(data) => data,
        }
    }
}

pub trait HttpFileResponse<'a>: HttpFile<'a> + Sized {
    fn respond_guard<T: From<FileData<'a>>>(
        &self,
        request: &http::Request<()>,
    ) -> Result<http::response::Builder, Result<http::Response<T>, http::Error>> {
        let method = request.method();
        if method != http::Method::HEAD
            && method != http::Method::OPTIONS
            && method != http::Method::GET
        {
            return Err(http::Response::builder()
                .status(http::StatusCode::METHOD_NOT_ALLOWED)
                .header(http::header::ALLOW, "GET, HEAD, OPTIONS")
                .body(FileData::from_static(&[]).into()));
        }
        match self.cache_busting() {
            CacheBusting::None => {}
            CacheBusting::Query(query_key) => {
                if let Some(res) = self.cachebust_uri(request.uri(), query_key) {
                    return Err(res);
                }
            }
            CacheBusting::Suffix(left_sep) => {
                if let Some(res) = self.cachebust_suffix(request.uri(), left_sep) {
                    return Err(res);
                }
            }
        }
        let mut response = self.response_headers(http::Response::builder());
        if method == http::Method::OPTIONS {
            response = response
                .status(http::StatusCode::NO_CONTENT)
                .header(http::header::ALLOW, "GET, HEAD, OPTIONS");
            return Err(response.body(FileData::from_static(&[]).into()));
        }
        if let Some(etag) = request
            .headers()
            .get(http::header::IF_NONE_MATCH)
            .and_then(|value| value.to_str().ok())
        {
            for esplit in etag.split(',') {
                let esplit = esplit.trim();
                if esplit == "*" || esplit == self.etag() {
                    return Err(response
                        .status(http::StatusCode::NOT_MODIFIED)
                        .body(FileData::from_static(&[]).into()));
                }
            }
        }
        if method == http::Method::HEAD {
            Err(response.body(FileData::from_static(&[]).into()))
        } else {
            Ok(response)
        }
    }

    fn respond<T: From<FileData<'a>>>(
        self,
        request: &http::Request<()>,
    ) -> Result<http::Response<T>, http::Error> {
        match self.respond_guard(request) {
            Ok(response) => response.body(T::from(self.into_data())),
            Err(res) => res,
        }
    }

    fn respond_borrowed<T: From<FileData<'a>>>(
        &self,
        request: &http::Request<()>,
    ) -> Result<http::Response<T>, http::Error> {
        match self.respond_guard(request) {
            Ok(response) => response.body(T::from(self.clone_data())),
            Err(res) => res,
        }
    }

    fn response_headers(&self, mut response: http::response::Builder) -> http::response::Builder {
        response = response
            .header(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_str(self.content_type()).unwrap(),
            )
            .header(
                http::header::ETAG,
                http::header::HeaderValue::from_str(self.etag()).unwrap(),
            );
        if self.cache_busting() != CacheBusting::None {
            response.header(
                http::header::CACHE_CONTROL,
                http::header::HeaderValue::from_static("public, max-age=31536000, immutable"),
            )
        } else {
            response.header(
                http::header::CACHE_CONTROL,
                http::header::HeaderValue::from_static("public, max-age=0, must-revalidate"),
            )
        }
    }

    /// Converts the file representation into a response.
    fn into_response<T: From<FileData<'a>>>(self) -> Result<http::Response<T>, http::Error> {
        self.response_headers(http::Response::builder())
            .body(T::from(self.into_data()))
    }

    /// Detects if the request needs to be redirected to a cache-busted URI. Used when the cache busting method is `CacheBusting::Query`.
    fn cachebust_uri<T: From<FileData<'a>>>(
        &self,
        old_uri: &http::Uri,
        query_key: &str,
    ) -> Option<Result<http::Response<T>, http::Error>> {
        if let Some(query) = old_uri.query() {
            let query_val = query.split('&').find_map(|pair| {
                let mut pair = pair.splitn(2, '=');
                if pair.next() == Some(query_key) {
                    pair.next()
                } else {
                    None
                }
            });
            let etag_str = self.etag_str();
            if query_val != Some(etag_str) {
                let old_path = old_uri.path();
                let mut new_path = String::with_capacity(
                    old_path.len() + 1 + query_key.len() + 1 + etag_str.len() + query.len(),
                );
                new_path.push_str(old_path);
                new_path.push('?');
                new_path.push_str(query_key);
                new_path.push('=');
                new_path.push_str(etag_str);
                if query_val.is_some() {
                    for x in query.split('&') {
                        if !x.starts_with(query_key)
                            || (x.len() > query_key.len() && !x[query_key.len()..].starts_with('='))
                        {
                            new_path.push('&');
                            new_path.push_str(x);
                        }
                    }
                } else if !query.is_empty() {
                    new_path.push('&');
                    new_path.push_str(query);
                }
                Some(
                    http::Response::builder()
                        .status(http::StatusCode::TEMPORARY_REDIRECT)
                        .header(http::header::LOCATION, new_path)
                        .body(FileData::from_static(&[]).into()),
                )
            } else {
                None
            }
        } else {
            let old_path = old_uri.path();
            let etag_str = self.etag_str();
            let mut new_path =
                String::with_capacity(old_path.len() + 1 + query_key.len() + 1 + etag_str.len());
            new_path.push_str(old_path);
            new_path.push('?');
            new_path.push_str(query_key);
            new_path.push('=');
            new_path.push_str(etag_str);
            Some(
                http::Response::builder()
                    .status(http::StatusCode::TEMPORARY_REDIRECT)
                    .header(http::header::LOCATION, new_path)
                    .body(FileData::from_static(&[]).into()),
            )
        }
    }

    /// Detects if the request needs to be redirected to a cache-busted URI. Used when the cache busting method is `CacheBusting::Suffix`.
    fn cachebust_suffix<T: From<FileData<'a>>>(
        &self,
        old_uri: &http::Uri,
        left_sep: Option<NonZeroU8>,
    ) -> Option<Result<http::Response<T>, http::Error>> {
        let old_path = old_uri.path();
        let etag_str = self.etag_str();
        if old_path.ends_with(etag_str) && old_path.len() > etag_str.len() {
            if let Some(left_sep) = left_sep {
                if old_path.as_bytes()[old_path.len() - etag_str.len() - 1] == left_sep.get() {
                    return None;
                }
            } else {
                return None;
            }
        }
        let ext = super::file_ext(old_path);
        let new_path = if let Some(ext) = ext {
            let basename = &old_path[..old_path.len() - ext.len() - 1];
            if basename.ends_with(etag_str) && basename.len() > etag_str.len() {
                if let Some(left_sep) = left_sep {
                    if basename.as_bytes()[basename.len() - etag_str.len() - 1] == left_sep.get() {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            // create new path in the form of `basename` + `left_sep` + `etag` + `ext`
            // the current basename may contain an etag, so we need to remove it
            let mut new_path =
                String::with_capacity(basename.len() + 1 + etag_str.len() + 1 + ext.len());
            new_path.push_str(basename);
            if let Some(left_sep) = left_sep {
                // remove left_sep and trailing from the basename appended into new_path
                if let Some(p) = basename.rfind(left_sep.get() as char) {
                    if basename.rfind('/').unwrap_or(0) < p {
                        new_path.truncate(p);
                    }
                }
                new_path.push(left_sep.get() as char);
            }
            new_path.push_str(etag_str);
            new_path.push('.');
            new_path.push_str(ext);
            new_path
        } else {
            let mut new_path = String::with_capacity(old_path.len() + 1 + etag_str.len());
            new_path.push_str(old_path);
            if let Some(left_sep) = left_sep {
                // remove left_sep and trailing from the basename appended into new_path
                if let Some(p) = old_path.rfind(left_sep.get() as char) {
                    if old_path.rfind('/').unwrap_or(0) < p {
                        new_path.truncate(p);
                    }
                }
                new_path.push(left_sep.get() as char);
            }
            new_path.push_str(etag_str);
            new_path
        };
        Some(
            http::Response::builder()
                .status(http::StatusCode::TEMPORARY_REDIRECT)
                .header(http::header::LOCATION, new_path)
                .body(FileData::from_static(&[]).into()),
        )
    }
}
