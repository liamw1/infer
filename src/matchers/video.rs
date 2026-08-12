/// Returns whether a buffer is M4V video data.
#[must_use]
pub fn is_m4v(buf: &[u8]) -> bool {
    buf.get(4..11) == Some(b"ftypM4V")
}

/// Returns whether a buffer is MKV video data.
#[must_use]
pub fn is_mkv(buf: &[u8]) -> bool {
    ebml_doctype(buf) == Some(b"matroska")
}

/// Returns whether a buffer is WEBM video data.
#[must_use]
pub fn is_webm(buf: &[u8]) -> bool {
    ebml_doctype(buf) == Some(b"webm")
}

/// Returns whether a buffer is Quicktime MOV video data.
#[must_use]
pub fn is_mov(buf: &[u8]) -> bool {
    matches!(buf.get(4..12), Some(b"ftypqt  "))
        || matches!(buf.get(4..8), Some(b"moov" | b"mdat"))
        || buf.get(12..16) == Some(b"mdat")
}

/// Returns whether a buffer is AVI video data.
#[must_use]
pub fn is_avi(buf: &[u8]) -> bool {
    buf.starts_with(b"RIFF") && buf.get(8..11) == Some(b"AVI")
}

/// Returns whether a buffer is WMV video data.
#[must_use]
pub fn is_wmv(buf: &[u8]) -> bool {
    buf.starts_with(b"\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9")
}

/// Returns whether a buffer is MPEG video data.
#[must_use]
pub fn is_mpeg(buf: &[u8]) -> bool {
    matches!(
        buf.first_chunk::<4>(),
        Some([0x00, 0x00, 0x01, 0xb0..=0xbf])
    )
}

/// Returns whether a buffer is FLV video data.
#[must_use]
pub fn is_flv(buf: &[u8]) -> bool {
    buf.starts_with(b"FLV\x01")
}

/// Returns whether a buffer is MP4 video data.
#[must_use]
pub fn is_mp4(buf: &[u8]) -> bool {
    matches!(buf.get(4..8), Some(b"ftyp"))
        && matches!(
            buf.get(8..12),
            Some(
                b"avc1"
                    | b"dash"
                    | b"iso2"
                    | b"iso3"
                    | b"iso4"
                    | b"iso5"
                    | b"iso6"
                    | b"isom"
                    | b"mmp4"
                    | b"mp41"
                    | b"mp42"
                    | b"mp4v"
                    | b"mp71"
                    | b"MSNV"
                    | b"NDAS"
                    | b"NDSC"
                    | b"NSDC"
                    | b"NDSH"
                    | b"NDSM"
                    | b"NDSP"
                    | b"NDSS"
                    | b"NDXC"
                    | b"NDXH"
                    | b"NDXM"
                    | b"NDXP"
                    | b"NDXS"
                    | b"F4V "
                    | b"F4P "
            )
        )
}

/// Reads an EBML variable-length integer, returning (value, bytes consumed).
fn read_vint(buf: &[u8]) -> Option<(u64, usize)> {
    let first = *buf.first()?;
    if first == 0 {
        return None; // Widths beyond 8 bytes unsupported
    }
    let width = first.leading_zeros() as usize + 1;
    let value = buf
        .get(1..width)?
        .iter()
        .fold(u64::from(first) & (0xFF >> width), |acc, &b| {
            acc << 8 | u64::from(b)
        });
    Some((value, width))
}

/// Walks the EBML header's child elements and returns the DocType value,
/// with trailing zero-padding stripped.
fn ebml_doctype(buf: &[u8]) -> Option<&[u8]> {
    let rest = buf.strip_prefix(b"\x1a\x45\xdf\xa3")?; // EBML header element ID
    let (header_len, consumed) = read_vint(rest)?;
    let header_len = usize::try_from(header_len).ok()?;
    let mut header = rest.get(consumed..consumed.checked_add(header_len)?)?;

    while !header.is_empty() {
        // Element ID: vint-shaped, but the marker bits are kept as part of the ID.
        let id_width = header.first()?.leading_zeros() as usize + 1;
        let (id, rest) = header.split_at_checked(id_width)?;

        let (len, consumed) = read_vint(rest)?;
        let len = usize::try_from(len).ok()?;
        let end = consumed.checked_add(len)?;
        let payload = rest.get(consumed..end)?;

        if id == [0x42, 0x82] {
            let pad = payload.iter().rev().take_while(|&&b| b == 0).count();
            return Some(&payload[..payload.len() - pad]);
        }
        header = &rest[end..];
    }
    None
}
