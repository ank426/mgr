use std::io::{self, Read};

// Reads image dimensions (width, height) by parsing format headers.
pub fn read_dimensions(mut reader: impl Read) -> io::Result<(u32, u32)> {
    let h = read_arr::<12>(&mut reader)?;

    if h[..3] == [0xFF, 0xD8, 0xFF] {
        jpeg_dimensions(&h[2..], &mut reader)
    } else if h[..4] == [0x89, 0x50, 0x4E, 0x47] {
        png_dimensions(&mut reader)
    } else if h[..4] == *b"RIFF" && h[8..12] == *b"WEBP" {
        webp_dimensions(&mut reader)
    } else {
        Err(invalid("unsupported image format"))
    }
}

// PNG: width/height as u32 BE at file offsets 16 and 20.
fn png_dimensions(reader: &mut impl Read) -> io::Result<(u32, u32)> {
    let b = read_arr::<12>(reader)?;
    Ok((u32_be(&b[4..]), u32_be(&b[8..])))
}

// WebP: dispatch on VP8 chunk variant (lossy, lossless, extended).
fn webp_dimensions(reader: &mut impl Read) -> io::Result<(u32, u32)> {
    // Chunk header: 4-byte tag ("VP8 "/"VP8L"/"VP8X") + 4-byte size
    match read_arr::<8>(reader)?[3] {
        b' ' => {
            // Lossy: width/height as u16 LE at chunk offsets 6 and 8 (lower 14 bits)
            let b = read_arr::<10>(reader)?;
            Ok(((u16_le(&b[6..]) & 0x3FFF) as u32, (u16_le(&b[8..]) & 0x3FFF) as u32))
        }
        b'L' => {
            // Lossless: 1 signature byte + 4 bytes bit-packed dimensions
            let b = read_arr::<5>(reader)?;
            let bits = u32_le(&b[1..]);
            Ok(((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1))
        }
        b'X' => {
            // Extended: 4 bytes flags + width-1 and height-1 as u24 LE
            let b = read_arr::<10>(reader)?;
            Ok((u24_le(&b[4..]) + 1, u24_le(&b[7..]) + 1))
        }
        _ => Err(invalid("unknown WebP variant")),
    }
}

// JPEG: scan markers until SOF. `prefix` contains unconsumed header bytes
// starting from the first marker (file offset 2).
fn jpeg_dimensions(prefix: &[u8], reader: &mut impl Read) -> io::Result<(u32, u32)> {
    let mut r = prefix.chain(reader);
    loop {
        let [0xFF, mut marker] = read_arr::<2>(&mut r)? else {
            return Err(invalid("invalid JPEG marker"));
        };
        // Skip fill bytes (consecutive FF padding between markers)
        while marker == 0xFF {
            [marker] = read_arr::<1>(&mut r)?;
        }
        match marker {
            // Byte stuffing (FF 00) — shouldn't occur pre-SOS but skip safely
            0x00 => continue,
            // Standalone markers (no length): TEM, RST0-7, SOI
            0x01 | 0xD0..=0xD8 => continue,
            // EOI / SOS — dimensions should have been found before these
            0xD9 => return Err(invalid("reached EOI without finding dimensions")),
            0xDA => return Err(invalid("reached SOS without finding dimensions")),
            // SOF markers contain dimensions
            0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {
                // length(2) + precision(1) + height(2) + width(2)
                let d = read_arr::<7>(&mut r)?;
                return Ok((u16_be(&d[5..]) as u32, u16_be(&d[3..]) as u32));
            }
            // All other markers: read length, skip body
            _ => {
                let len = u16_be(&read_arr::<2>(&mut r)?) as u64;
                if len < 2 {
                    return Err(invalid("invalid JPEG marker length"));
                }
                if io::copy(&mut r.by_ref().take(len - 2), &mut io::sink())? < len - 2 {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
            }
        }
    }
}

fn read_arr<const N: usize>(reader: &mut impl Read) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn u32_be(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}
fn u16_be(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn u32_le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}
fn u24_le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], 0])
}
fn u16_le(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}

fn invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}
