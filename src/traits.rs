use bytedata::ByteData;

use crate::CacheBusting;

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
    fn cache_busting(&self) -> &CacheBusting {
        &CacheBusting::None
    }
    /// Extracts the data of the file.
    fn into_data(self) -> ByteData<'a>;
    /// Clones the data of the file. This may only copy the reference.
    fn clone_data(&self) -> ByteData<'a>;
}
