use core::hint::assert_unchecked;

/// Encodes the first part of the input bytes as URL-encoded bytes,
/// returning a tuple of the encoded part and the remaining unencoded part.
///
/// The encoded part will either contain a safe slice of the input,
/// or a chunk of URL-encoded bytes.
pub fn urlencode_iter_fn<'a>(
    mut input: bytedata::ByteData<'a>,
) -> (bytedata::StringData<'a>, bytedata::ByteData<'a>) {
    if input.is_empty() {
        return (bytedata::StringData::empty(), input);
    }
    let bytes = input.as_slice();
    let mut i = 0;
    loop {
        unsafe { assert_unchecked(i <= bytes.len()) };
        if i == bytes.len() {
            return (
                unsafe { bytedata::StringData::from_bytedata_unchecked(input) },
                bytedata::ByteData::empty(),
            );
        }
        let b = bytes[i];
        let is_safe =
            matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~');
        if is_safe {
            i += 1;
            continue;
        }
        if i != 0 {
            let (a, b) = input.split_at(i);
            return (
                unsafe { bytedata::StringData::from_bytedata_unchecked(a) },
                b,
            );
        }
        break;
    }
    let mut encoded = [0u8; 12];
    let mut enc_i = 0;
    loop {
        unsafe { assert_unchecked(i < bytes.len()) };
        unsafe { assert_unchecked(enc_i < encoded.len()) };
        let b = bytes[i];
        encoded[enc_i] = b'%';
        encoded[enc_i + 1] = match (b >> 4) & 0x0F {
            v @ 0..=9 => b'0' + v,
            v @ 10..=15 => b'A' + (v - 10),
            _ => unreachable!(),
        };
        encoded[enc_i + 2] = match b & 0x0F {
            v @ 0..=9 => b'0' + v,
            v @ 10..=15 => b'A' + (v - 10),
            _ => unreachable!(),
        };
        enc_i += 3;
        i += 1;
        if enc_i == 12
            || i == bytes.len()
            || matches!(bytes[i], b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~')
        {
            input.make_sliced(i..);
            return (
                unsafe {
                    bytedata::StringData::from_bytedata_unchecked(
                        bytedata::ByteData::from_chunk_slice(bytedata::const_slice_unchecked(
                            encoded.as_slice(),
                            0..enc_i,
                        )),
                    )
                },
                input,
            );
        }
    }
}

/// URL-encodes the given input bytes, returning a `StringQueue` containing the encoded data.
#[inline]
pub fn urlencode<'a>(input: bytedata::ByteData<'a>) -> bytedata::StringQueue<'a> {
    let mut builder = bytedata::StringQueue::new();
    urlencode_into(input, &mut builder);
    builder
}
/// URL-encodes the given input bytes in a queue, returning a `StringQueue` containing the encoded data.
#[inline]
pub fn urlencode_queue<'a>(input: bytedata::ByteQueue<'a>) -> bytedata::StringQueue<'a> {
    let mut builder = bytedata::StringQueue::new();
    for chunk in input.into_iter() {
        urlencode_into(chunk, &mut builder);
    }
    builder
}
/// URL-encodes the given input bytes into the provided `StringQueue` builder.
#[inline]
pub fn urlencode_into<'a>(
    mut input: bytedata::ByteData<'a>,
    builder: &mut bytedata::StringQueue<'a>,
) {
    while !input.is_empty() {
        let (encoded, remaining) = urlencode_iter_fn(input);
        builder.push_back(encoded);
        input = remaining;
    }
}
/// URL-encodes the given input bytes in a queue into the provided `StringQueue` builder.
#[inline]
pub fn urlencode_queue_into<'a>(
    input: bytedata::ByteQueue<'a>,
    builder: &mut bytedata::StringQueue<'a>,
) {
    for chunk in input.into_iter() {
        urlencode_into(chunk, builder);
    }
}

/// Decodes the first part of the input bytes from URL-encoded bytes,
/// returning a tuple of the decoded part and the remaining undecoded part.
pub fn urldecode_iter_fn<'a>(
    mut input: bytedata::ByteData<'a>,
) -> (bytedata::ByteData<'a>, bytedata::ByteData<'a>) {
    if input.is_empty() {
        return (bytedata::ByteData::empty(), input);
    }
    let bytes = input.as_slice();
    let mut i = 0;
    loop {
        unsafe { assert_unchecked(i <= bytes.len()) };
        if i == bytes.len() {
            return (input, bytedata::ByteData::empty());
        }
        let b = bytes[i];
        if b != b'%' {
            i += 1;
            continue;
        }
        if i != 0 {
            return input.split_at(i);
        }
        break;
    }

    let mut decoded = [0u8; 14];
    let mut dec_i = 0;
    loop {
        if i + 2 >= bytes.len() {
            break;
        }
        unsafe { assert_unchecked(dec_i < decoded.len()) };
        let hi = bytes[i + 1];
        let lo = bytes[i + 2];
        let hi_val = match hi {
            b'0'..=b'9' => hi - b'0',
            b'A'..=b'F' => hi - b'A' + 10,
            b'a'..=b'f' => hi - b'a' + 10,
            _ => break,
        };
        let lo_val = match lo {
            b'0'..=b'9' => lo - b'0',
            b'A'..=b'F' => lo - b'A' + 10,
            b'a'..=b'f' => lo - b'a' + 10,
            _ => break,
        };
        decoded[dec_i] = (hi_val << 4) | lo_val;
        dec_i += 1;
        i += 3;
        if i + 2 > bytes.len() || bytes[i] != b'%' || dec_i == decoded.len() {
            break;
        }
    }
    if i == 0 {
        return (bytedata::ByteData::empty(), input);
    }
    input.make_sliced(i..);
    return (
        bytedata::ByteData::from_chunk_slice(&decoded[0..dec_i]),
        input,
    );
}

/// URL-decodes the given input bytes, returning a `ByteQueue` containing the decoded data.
#[inline]
pub fn urldecode<'a>(
    input: bytedata::ByteData<'a>,
) -> Result<bytedata::ByteQueue<'a>, (bytedata::ByteQueue<'a>, bytedata::ByteData<'a>)> {
    let mut builder = bytedata::ByteQueue::new();
    match urldecode_into(input, &mut builder) {
        Ok(()) => Ok(builder),
        Err(e) => Err((builder, e)),
    }
}
/// URL-decodes the given input bytes in a queue, returning a `ByteQueue` containing the decoded data.
#[inline]
pub fn urldecode_queue<'a>(
    input: bytedata::ByteQueue<'a>,
) -> Result<bytedata::ByteQueue<'a>, (bytedata::ByteQueue<'a>, bytedata::ByteQueue<'a>)> {
    let mut builder = bytedata::ByteQueue::new();
    match urldecode_queue_into(input, &mut builder) {
        Ok(()) => Ok(builder),
        Err(e) => Err((builder, e)),
    }
}
/// URL-decodes the given input bytes into the provided `ByteQueue` builder.
#[inline]
pub fn urldecode_into<'a>(
    mut input: bytedata::ByteData<'a>,
    builder: &mut bytedata::ByteQueue<'a>,
) -> Result<(), bytedata::ByteData<'a>> {
    while !input.is_empty() {
        let (decoded, remaining) = urldecode_iter_fn(input);
        if decoded.is_empty() {
            return Err(remaining);
        }
        builder.push_back(decoded);
        input = remaining;
    }
    Ok(())
}
/// URL-decodes the given input bytes in a queue into the provided `ByteQueue` builder.
#[inline]
pub fn urldecode_queue_into<'a>(
    mut input: bytedata::ByteQueue<'a>,
    builder: &mut bytedata::ByteQueue<'a>,
) -> Result<(), bytedata::ByteQueue<'a>> {
    while let Some(chunk) = input.pop_front() {
        if let Err(e) = urldecode_into(chunk, builder) {
            input.push_front(e);
            return Err(input);
        }
    }
    Ok(())
}
