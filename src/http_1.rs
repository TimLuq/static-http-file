use core::num::NonZeroU8;
use alloc::string::String;

use bytedata::ByteData;

use http_1 as http;

use crate::{CacheBusting, HttpFile};

pub trait HttpFileResponse<'a>: HttpFile<'a> + Sized {
    fn respond_guard<T: From<ByteData<'a>>>(
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
                .body(ByteData::from_static(&[]).into()));
        }
        match self.cache_busting() {
            CacheBusting::None => {}
            CacheBusting::Query(query_key) => {
                if let Some(res) = self.cachebust_uri(request.uri(), query_key.as_str()) {
                    return Err(res);
                }
            }
            CacheBusting::Suffix(left_sep) => {
                if let Some(res) = self.cachebust_suffix(request.uri(), *left_sep) {
                    return Err(res);
                }
            }
        }
        let mut response = self.response_headers(http::Response::builder());
        if method == http::Method::OPTIONS {
            response = response
                .status(http::StatusCode::NO_CONTENT)
                .header(http::header::ALLOW, "GET, HEAD, OPTIONS");
            return Err(response.body(ByteData::from_static(&[]).into()));
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
                        .body(ByteData::from_static(&[]).into()));
                }
            }
        }
        if method == http::Method::HEAD {
            Err(response.body(ByteData::from_static(&[]).into()))
        } else {
            Ok(response)
        }
    }

    fn respond<T: From<ByteData<'a>>>(
        self,
        request: &http::Request<()>,
    ) -> Result<http::Response<T>, http::Error> {
        match self.respond_guard(request) {
            Ok(response) => response.body(T::from(self.into_data())),
            Err(res) => res,
        }
    }

    fn respond_borrowed<T: From<ByteData<'a>>>(
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
        if !matches!(self.cache_busting(), CacheBusting::None) {
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
    fn into_response<T: From<ByteData<'a>>>(self) -> Result<http::Response<T>, http::Error> {
        self.response_headers(http::Response::builder())
            .body(T::from(self.into_data()))
    }

    /// Detects if the request needs to be redirected to a cache-busted URI. Used when the cache busting method is `CacheBusting::Query`.
    fn cachebust_uri<T: From<ByteData<'a>>>(
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
                        .body(ByteData::from_static(&[]).into()),
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
                    .body(ByteData::from_static(&[]).into()),
            )
        }
    }

    /// Detects if the request needs to be redirected to a cache-busted URI. Used when the cache busting method is `CacheBusting::Suffix`.
    fn cachebust_suffix<T: From<ByteData<'a>>>(
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
                .body(ByteData::from_static(&[]).into()),
        )
    }
}