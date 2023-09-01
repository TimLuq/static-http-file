use alloc::{vec::Vec, boxed::Box};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum CacheBusting {
    #[default]
    None,
    Query,
    Suffix,
}

pub trait HttpFile<'a> {
    fn content_type(&self) -> &str;
    fn data(&self) -> &[u8];
    fn etag(&self) -> &str;
    fn cache_busting(&self) -> CacheBusting {
        CacheBusting::None
    }
    fn into_data(self) -> FileData<'a>;
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
    fn response_headers(&self, mut response: http::response::Builder) -> http::response::Builder {
        response = response.header(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_str(self.content_type()).unwrap(),
        ).header(
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
    fn into_response<T: From<FileData<'a>>>(self) -> Result<http::Response<T>, http::Error> {
        self.response_headers(http::Response::builder()).body(T::from(self.into_data()))
    }
}
