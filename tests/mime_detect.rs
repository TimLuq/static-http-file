use static_http_file::{detect_mime_type_magic, detect_mime_type_ext};

static DETECT_MIME_TYPE: &[(&str, &str)] = &[
    ("./assets/pixel/pixel.avif", "image/avif"),
    ("./assets/pixel/pixel.bmp", "image/bmp"),
    ("./assets/pixel/pixel.eps", "application/eps"),
    ("./assets/pixel/pixel.gif", "image/gif"),
    ("./assets/pixel/pixel.heic", "image/heic"),
    ("./assets/pixel/pixel.ico", "image/vnd.microsoft.icon"),
    ("./assets/pixel/pixel.jpg", "image/jpeg"),
    ("./assets/pixel/pixel.pdf", "application/pdf"),
    ("./assets/pixel/pixel.png", "image/png"),
    ("./assets/pixel/pixel.ps", "application/postscript"),
    ("./assets/pixel/pixel.tif", "image/tiff"),
    ("./assets/pixel/pixel.webp", "image/webp"),
];

#[test]
fn test_detect_mime_type_pixel() {
    for (path, mime) in DETECT_MIME_TYPE {
        let data = std::fs::read(path).unwrap();
        let mime_path = detect_mime_type_ext(path);
        assert_eq!(mime_path, Some(*mime));
        let mime_type = detect_mime_type_magic(&data);
        assert_eq!(mime_type, Some(*mime));
    };
}
