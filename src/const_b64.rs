const BASE64URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode a byte slice as base64url.
/// The output buffer size `S` must be at least 4/3 the size of the input otherwise this function will panic.
/// The returned offset is the number of bytes written to the output buffer.
///
/// This can be used in constant contexts when the input is a constant byte slice of a known length.
pub const fn b64url_const<const S: usize>(
    data: &[u8],
    mut trg: [u8; S],
    offset: usize,
) -> ([u8; S], usize) {
    if offset >= S {
        panic!("Offset too large");
    }
    let inp_len = data.len();
    let out_len = S - offset;
    if out_len < (4 * inp_len) / 3 {
        panic!("Output buffer too small");
    }
    let mut i = 0;
    let mut o = offset;
    while inp_len - i >= 3 {
        let b0 = data[i];
        let b1 = data[i + 1];
        let b2 = data[i + 2];
        trg[o] = BASE64URL[(b0 >> 2) as usize];
        trg[o + 1] = BASE64URL[(((b0 & 0b0011) << 4) | (b1 >> 4)) as usize];
        trg[o + 2] = BASE64URL[(((b1 & 0b1111) << 2) | (b2 >> 6)) as usize];
        trg[o + 3] = BASE64URL[(b2 & 0b111111) as usize];
        i += 3;
        o += 4;
    }
    let o = match inp_len - i {
        1 => {
            let b0 = data[i];
            trg[o] = BASE64URL[(b0 >> 2) as usize];
            trg[o + 1] = BASE64URL[((b0 & 0b0011) << 4) as usize];
            o + 2
        }
        2 => {
            let b0 = data[i];
            let b1 = data[i + 1];
            trg[o] = BASE64URL[(b0 >> 2) as usize];
            trg[o + 1] = BASE64URL[(((b0 & 0b0011) << 4) | (b1 >> 4)) as usize];
            trg[o + 2] = BASE64URL[((b1 & 0b1111) << 2) as usize];
            o + 3
        }
        _ => o,
    };
    (trg, o)
}
