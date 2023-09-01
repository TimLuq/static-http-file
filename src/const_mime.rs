/// Detects the mime type of a file based on its extension or magic bytes.
pub const fn detect_mime_type(path: &str, data: &[u8]) -> Option<&'static str> {
    let ext = detect_mime_type_ext(path);
    if ext.is_some() {
        return ext;
    }
    detect_mime_type_magic(data)
}

/// Returns the extension of a file, if any is found.
pub const fn file_ext(path: &'_ str) -> Option<&'_ str> {
    let pathb = path.as_bytes();
    let mut i = pathb.len();
    loop {
        if i == 0 {
            return None;
        }
        let i2 = i - 1;
        let b = pathb[i2];
        if b == b'.' {
            return Some(unsafe {
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                    pathb.as_ptr().add(i),
                    pathb.len() - i,
                ))
            });
        }
        if b != b'/' && b != b'\\' {
            i = i2;
            continue;
        }
        return None;
    }
}

/// Detects the mime type of a file based on its extension.
pub const fn detect_mime_type_ext(path: &str) -> Option<&'static str> {
    let Some(ext) = file_ext(path) else {
        return None;
    };
    match ext.as_bytes() {
        // common web formats
        b"css" => Some("text/css"),
        b"html" | b"htm" => Some("text/html"),
        b"js" | b"mjs" => Some("application/javascript"),
        b"json" => Some("application/json"),
        b"jsonld" => Some("application/ld+json"),
        b"wasm" => Some("application/wasm"),
        b"webmanifest" => Some("application/manifest+json"),
        b"xhtml" => Some("application/xhtml+xml"),

        // config files
        b"yaml" | b"yml" => Some("application/x-yaml"),
        b"toml" => Some("application/toml"),
        b"ini" => Some("text/plain"),

        // shell files
        b"sh" => Some("application/x-sh"),
        b"bat" => Some("application/x-bat"),
        b"cmd" => Some("application/x-cmd"),

        // image types
        b"avif" => Some("image/avif"),
        b"apng" => Some("image/apng"),
        b"bmp" => Some("image/bmp"),
        b"png" => Some("image/png"),
        b"jpg" | b"jpeg" => Some("image/jpeg"),
        b"gif" => Some("image/gif"),
        b"ico" => Some("image/vnd.microsoft.icon"),
        b"svg" => Some("image/svg+xml"),
        b"tiff" | b"tif" => Some("image/tiff"),
        b"webp" => Some("image/webp"),

        // fonts
        b"eot" => Some("application/vnd.ms-fontobject"),
        b"otf" => Some("font/otf"),
        b"ttf" => Some("font/ttf"),
        b"woff" => Some("font/woff"),
        b"woff2" => Some("font/woff2"),

        // documents
        b"atom" => Some("application/atom+xml"),
        b"csv" => Some("text/csv"),
        b"doc" => Some("application/msword"),
        b"docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        b"ics" => Some("text/calendar"),
        b"md" => Some("text/markdown"),
        b"odp" => Some("application/vnd.oasis.opendocument.presentation"),
        b"ods" => Some("application/vnd.oasis.opendocument.spreadsheet"),
        b"odt" => Some("application/vnd.oasis.opendocument.text"),
        b"pdf" => Some("application/pdf"),
        b"ppt" => Some("application/vnd.ms-powerpoint"),
        b"pptx" => {
            Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
        }
        b"rss" => Some("application/rss+xml"),
        b"rtf" => Some("application/rtf"),
        b"txt" => Some("text/plain"),
        b"vsd" => Some("application/vnd.visio"),
        b"xls" => Some("application/vnd.ms-excel"),
        b"xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        b"xml" => Some("application/xml"),

        // comressed/archived
        b"7z" => Some("application/x-7z-compressed"),
        b"bz2" => Some("application/x-bzip2"),
        b"gz" => Some("application/gzip"),
        b"jar" => Some("application/java-archive"),
        b"mpkg" => Some("application/vnd.apple.installer+xml"),
        b"rar" => Some("application/vnd.rar"),
        b"tar" => Some("application/x-tar"),
        b"war" => Some("application/java-archive"),
        b"xz" => Some("application/x-xz"),
        b"zip" => Some("application/zip"),

        // audio
        b"aac" => Some("audio/aac"),
        b"flac" => Some("audio/flac"),
        b"m4a" => Some("audio/mp4"),
        b"mid" | b"midi" => Some("audio/midi"),
        b"mp3" => Some("audio/mpeg"),
        b"oga" => Some("audio/ogg"),
        b"opus" => Some("audio/opus"),
        b"wav" => Some("audio/wav"),
        b"weba" => Some("audio/webm"),

        // video
        b"mp4" | b"m4v" => Some("video/mp4"),
        b"mpeg" | b"mpg" => Some("video/mpeg"),
        b"mkv" => Some("video/x-matroska"),
        b"webm" => Some("video/webm"),

        // media containers
        b"m3u8" => Some("application/x-mpegURL"),
        b"ogg" | b"ogx" => Some("application/ogg"),

        _ => None,
    }
}

type MagicLookup = (MagicOffset, &'static [u8], Magic);

enum Magic {
    Mime(&'static str),
    Specialized(Option<&'static str>, &'static [MagicLookup]),
}

enum MagicOffset {
    At(usize),
    Before(usize),
}

const FTYP: &[MagicLookup] = &[
    (MagicOffset::At(4), b"avif", Magic::Mime("image/avif")),
    (MagicOffset::At(4), b"heic", Magic::Mime("image/heic")),
    (MagicOffset::At(4), b"isom", Magic::Mime("video/mp4")),
    (MagicOffset::At(4), b"mp41", Magic::Mime("video/mp4")),
    (MagicOffset::At(4), b"mp42", Magic::Mime("video/mp4")),
    (MagicOffset::At(4), b"mmp4", Magic::Mime("video/mp4")),
    (MagicOffset::At(4), b"M4A", Magic::Mime("audio/mp4")),
];

const RIFF: &[MagicLookup] = &[
    (MagicOffset::At(4), b"AVI ", Magic::Mime("video/x-msvideo")),
    (MagicOffset::At(4), b"CDDA", Magic::Mime("audio/aiff")),
    (MagicOffset::At(4), b"WAVE", Magic::Mime("audio/wav")),
    (MagicOffset::At(4), b"WEBP", Magic::Mime("image/webp")),
];

const XML: &[MagicLookup] = &[
    (
        MagicOffset::Before(46),
        b"<!DOCTYPE html",
        Magic::Mime("application/xhtml+xml"),
    ),
    (
        MagicOffset::Before(46),
        b"<!DOCTYPE svg",
        Magic::Mime("image/svg+xml"),
    ),
    (
        MagicOffset::Before(120),
        b"xmlns=\"http://www.w3.org/1999/html\"",
        Magic::Mime("application/xhtml+xml"),
    ),
    (
        MagicOffset::Before(120),
        b"xmlns=\"http://www.w3.org/2000/svg\"",
        Magic::Mime("image/svg+xml"),
    ),
    (
        MagicOffset::Before(46),
        b"<html",
        Magic::Mime("application/xhtml+xml"),
    ),
    (
        MagicOffset::Before(46),
        b"<svg",
        Magic::Mime("image/svg+xml"),
    ),
];

const MAGICS: &[MagicLookup] = &[
    (
        MagicOffset::At(0),
        b"\0\0\x01\xBA",
        Magic::Mime("video/mpeg"),
    ),
    (
        MagicOffset::At(0),
        b"\0\0\x01\xBB",
        Magic::Mime("video/mpeg"),
    ),
    (MagicOffset::At(0), b"\0asm", Magic::Mime("text/x-asm")),
    (
        MagicOffset::At(0),
        b"\x1A\x45\xDF\xA3",
        Magic::Mime("video/webm"),
    ),
    (
        MagicOffset::At(0),
        b"\x1F\x8B\x08",
        Magic::Mime("application/x-gzip"),
    ),
    (
        MagicOffset::At(0),
        b"#!/bin/bash\n",
        Magic::Mime("application/x-sh"),
    ),
    (
        MagicOffset::At(0),
        b"#!/bin/sh\n",
        Magic::Mime("application/x-sh"),
    ),
    (
        MagicOffset::At(0),
        b"7z\xBC\xAF\x27\x1C",
        Magic::Mime("application/x-7z-compressed"),
    ),
    (
        MagicOffset::At(0),
        b"<?xml",
        Magic::Specialized(Some("text/xml"), XML),
    ),
    (
        MagicOffset::At(0),
        b"<!DOCTYPE html",
        Magic::Mime("text/html"),
    ),
    (MagicOffset::At(0), b"<html", Magic::Mime("text/html")),
    (MagicOffset::At(0), b"<svg", Magic::Mime("image/svg+xml")),
    (
        MagicOffset::At(0),
        b"BZh",
        Magic::Mime("application/x-bzip2"),
    ),
    (MagicOffset::At(0), b"GIF87a", Magic::Mime("image/gif")),
    (MagicOffset::At(0), b"GIF89a", Magic::Mime("image/gif")),
    (MagicOffset::At(0), b"I I", Magic::Mime("image/tiff")),
    (MagicOffset::At(0), b"ID3", Magic::Mime("audio/mp3")),
    (MagicOffset::At(0), b"II*\0", Magic::Mime("image/tiff")),
    (MagicOffset::At(0), b"MM\0*", Magic::Mime("image/tiff")),
    (MagicOffset::At(0), b"MM\0+", Magic::Mime("image/tiff")),
    (MagicOffset::At(0), b"MThd", Magic::Mime("audio/midi")),
    (
        MagicOffset::At(0),
        b"OggS\0\x02\0\0\0\0\0\0\0\0",
        Magic::Mime("application/ogg"),
    ),
    (
        MagicOffset::At(0),
        b"PK\x03\x04",
        Magic::Mime("application/ogg"),
    ),
    (MagicOffset::At(0), b"RIFF", Magic::Specialized(None, RIFF)),
    (
        MagicOffset::At(0),
        b"Rar!\x1A\x07",
        Magic::Mime("application/vnd.rar"),
    ),
    (MagicOffset::At(0), b"gimp xcf ", Magic::Mime("image/x-xcf")),
    (MagicOffset::At(0), b"icns", Magic::Mime("image/x-icns")),
    (MagicOffset::At(0), b"true\0", Magic::Mime("font/ttf")),
    (MagicOffset::At(0), b"wOFF", Magic::Mime("font/woff")),
    (MagicOffset::At(0), b"wOF2", Magic::Mime("font/woff2")),
    (MagicOffset::At(0), b"%PDF-", Magic::Mime("application/pdf")),
    (
        MagicOffset::At(0),
        b"%PNG\x0D\x0A\x1A\x0A",
        Magic::Mime("image/png"),
    ),
    (MagicOffset::At(0), b"\xFF\xD8", Magic::Mime("image/jpeg")),
    (MagicOffset::At(4), b"ftyp", Magic::Specialized(None, FTYP)),
    (MagicOffset::At(4), b"moov", Magic::Mime("video/quicktime")),
    (
        MagicOffset::At(257),
        b"ustar",
        Magic::Mime("application/x-tar"),
    ),
];

/// Detects the mime type of a file based on its magic bytes.
pub const fn detect_mime_type_magic(data: &[u8]) -> Option<&'static str> {
    let data_len = data.len();
    if data_len != 0 {
        let data_ptr = data.as_ptr();
        lookup_magic(MAGICS, data_len, data_ptr)
    } else {
        None
    }
}

const fn lookup_magic(
    magics: &[MagicLookup],
    data_len: usize,
    data_ptr: *const u8,
) -> Option<&'static str> {
    let mut i = 0;
    loop {
        if i == magics.len() {
            return None;
        }
        let (offset, magic, magic_type) = &magics[i];
        match offset {
            MagicOffset::At(offset) if *offset + magic.len() > data_len => {
                i += 1;
                continue;
            }
            MagicOffset::At(offset) => {
                if !bytes_matches(unsafe { data_ptr.add(*offset) }, magic) {
                    i += 1;
                    continue;
                }
            }
            MagicOffset::Before(_) if magic.len() > data_len => {
                i += 1;
                continue;
            }
            MagicOffset::Before(offset) => {
                let offset = *offset;
                let mx = data_len - magic.len();
                let mx = if offset < mx { offset } else { mx };
                let mut j = 0;
                let is_matching = loop {
                    if !bytes_matches(unsafe { data_ptr.add(j) }, magic) {
                        j += 1;
                        if j != mx {
                            continue;
                        } else {
                            break false;
                        }
                    }
                    break true;
                };
                if !is_matching {
                    i += 1;
                    continue;
                }
            }
        }
        match magic_type {
            Magic::Mime(mime) => {
                return Some(mime);
            }
            Magic::Specialized(mime, magics) => {
                let r = lookup_magic(magics, data_len, data_ptr);
                if r.is_some() {
                    return r;
                }
                return *mime;
            }
        }
    }
}

const fn bytes_matches(lhs: *const u8, rhs: &[u8]) -> bool {
    let mut i = 0;
    loop {
        if i == rhs.len() {
            return true;
        }
        if unsafe { *lhs.add(i) } != rhs[i] {
            return false;
        }
        i += 1;
    }
}
