#[test]
fn test_detect_mime_type_ext() {
    use crate::detect_mime_type_ext;

    assert_eq!(detect_mime_type_ext("foo.txt"), Some("text/plain"));
    assert_eq!(detect_mime_type_ext("foo.html"), Some("text/html"));
    assert_eq!(detect_mime_type_ext("foo.css"), Some("text/css"));
    assert_eq!(
        detect_mime_type_ext("foo.js"),
        Some("application/javascript")
    );
    assert_eq!(detect_mime_type_ext("foo.json"), Some("application/json"));
    assert_eq!(detect_mime_type_ext("foo.svg"), Some("image/svg+xml"));
    assert_eq!(detect_mime_type_ext("foo.png"), Some("image/png"));

    assert_eq!(detect_mime_type_ext("foo"), None);
    assert_eq!(detect_mime_type_ext("foo."), None);
    assert_eq!(detect_mime_type_ext("foo.js/test"), None);
    assert_eq!(detect_mime_type_ext("foo/js"), None);
}

#[test]
fn test_detect_mime_type_magic() {
    use crate::detect_mime_type_magic;

    assert_eq!(detect_mime_type_magic(b"<html></html>"), Some("text/html"));
    assert_eq!(
        detect_mime_type_magic(b"<!DOCTYPE html><title>test"),
        Some("text/html")
    );

    assert_eq!(
        detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<!DOCTYPE html>"),
        Some("application/xhtml+xml")
    );
    assert_eq!(detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<html xmlns=\"http://www.w3.org/1999/html\"></html>"), Some("application/xhtml+xml"));
    assert_eq!(detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<div xmlns=\"http://www.w3.org/1999/html\"></div>"), Some("application/xhtml+xml"));

    assert_eq!(
        detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<svg></svg>"),
        Some("image/svg+xml")
    );
    assert_eq!(detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"), Some("image/svg+xml"));
    assert_eq!(detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<g xmlns=\"http://www.w3.org/2000/svg\"></g>"), Some("image/svg+xml"));

    assert_eq!(
        detect_mime_type_magic(b"<?xml version=\"1.0\" encoding=\"utf-8\">\n<g></g>"),
        Some("text/xml")
    );
}

#[test]
fn test_const_http_file() {
    use crate::const_http_file;

    let file = const_http_file!("../.gitignore");
    assert_eq!(file.mime, "application/octet-data");
    assert_eq!(file.etag.len(), 12);
    assert_eq!(file.data.len(), 20);
}
