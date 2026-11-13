//! Small image helpers shared by avatar and logo uploads.

/// Base64-encode bytes without pulling in a crate. Used for `data:` URLs.
pub fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        out.push(TABLE[(a >> 2) as usize] as char);
        out.push(TABLE[((a & 0b11) << 4 | b >> 4) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((b & 0b1111) << 2 | c >> 6) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(c & 0b111111) as usize] as char
        } else {
            '='
        });
    }
    out
}

pub fn data_url(content_type: &str, bytes: &[u8]) -> String {
    format!("data:{content_type};base64,{}", base64_encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encodes_known_values() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_pads_remaining_bytes() {
        assert!(base64_encode(&[0]).ends_with("=="));
        assert!(base64_encode(&[0, 0]).ends_with("="));
        assert!(!base64_encode(&[0, 0, 0]).ends_with('='));
    }

    #[test]
    fn data_url_includes_media_type() {
        assert_eq!(data_url("image/png", b"foo"), "data:image/png;base64,Zm9v");
    }
}
