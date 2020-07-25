#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct UUID(u128);


fn is_ascii_hex_character(c: u8) -> bool {
    (b'0' <= c && c <= b'9') || (b'a' <= c && c <= b'f')
}


pub fn parse_uuid(raw: &[u8]) -> Option<UUID> {
    use std::mem::size_of;

    let mut result = UUID(0);
    let mut result_i = 0;

    let mut raw_i = 0;
    while result_i < size_of::<UUID>() && raw_i + 1 < raw.len() {
        if is_ascii_hex_character(raw[raw_i]) &&
                is_ascii_hex_character(raw[raw_i + 1]) {
            let pair = unsafe {
                // This is safe because we just verified both characters are
                // within the lower ASCII range.
                std::str::from_utf8_unchecked(&raw[raw_i..raw_i + 2])
            };

            // Likewise, we just verified that these are two hex characters,
            // which will never overflow 128 bits (won't even overflow 8)...
            let offset = (size_of::<UUID>() - result_i - 1) * 8;
            result.0 |= u128::from_str_radix(pair, 16).unwrap() << offset;

            result_i += 1;
            raw_i += 2;
        } else if raw[raw_i] == b'-' {
            raw_i += 1;
        } else {
            raw_i += 2;
        }
    }

    if result_i == size_of::<UUID>() && raw_i >= raw.len() {
        Some(UUID(result.0))
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use crate::parse::{UUID, parse_uuid};

    #[test]
    fn standard_format() {
        let raw_uuid = b"72aee7fd-3842-49cf-a196-be874a72ed9c";
        let uuid = parse_uuid(raw_uuid);
        assert!(uuid.is_some());

        assert_eq!(uuid.unwrap(),
                   UUID(152440156471620327380106519705898118556));
    }

    #[test]
    fn missing_dashes() {
        let raw_uuid = b"72aee7fd384249cfa196be874a72ed9c";
        let uuid = parse_uuid(raw_uuid);
        assert!(uuid.is_some());

        assert_eq!(uuid.unwrap(),
                   UUID(152440156471620327380106519705898118556));
    }

    #[test]
    fn empty() {
        let raw_uuid = b"";
        assert!(parse_uuid(raw_uuid).is_none());
    }

    #[test]
    fn non_dashes() {
        let raw_uuid = b"72aee7fd_3842-49cf-a196-be874a72ed9c";
        assert!(parse_uuid(raw_uuid).is_none());
    }

    #[test]
    fn truncated() {
        let raw_uuid = b"72aee7fd-3842-49cf-a196-be874a72ed";
        assert!(parse_uuid(raw_uuid).is_none());
    }

    #[test]
    fn too_long() {
        let raw_uuid = b"72aee7fd-3842-49cf-a196-be874a72ed9c\
                         72aee7fd-3842-49cf-a196-be874a72ed9c";
        assert!(parse_uuid(raw_uuid).is_none());
    }
}
