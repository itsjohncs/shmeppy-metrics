use memchr::memchr;

use std::fmt;


#[derive(Debug)]
pub struct Parts<'a> {
    pub uuid: &'a[u8],
    pub timestamp: &'a[u8],
    pub message: &'a[u8],
}


impl<'a> fmt::Display for Parts<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::str::from_utf8;
        write!(f, "({}, {}, {})",
               from_utf8(self.uuid).unwrap_or("<invalid utf8>"),
               from_utf8(self.timestamp).unwrap_or("<invalid utf8>"),
               from_utf8(self.message).unwrap_or("<invalid utf8>"))
    }
}


/**
 * Quickly find the important bits of a log message.
 *
 * This returns slices of the original log line, so there's no copying
 * that takes place.
 */
pub fn locate_parts(log_line: &[u8]) -> Option<Parts> {
    // The UUID is between the first pair of parents in the log line
    let left_paren = memchr(b'(', log_line)?;
    let right_paren = left_paren + memchr(b')', &log_line[left_paren..])?;

    // The datetime is between the first dash after the UUID and the right
    // square bracket. The message is everything after the right bracket.
    let dash = right_paren + memchr(b'-', &log_line[right_paren..])?;
    let right_bracket = dash + memchr(b']', &log_line[dash..])?;

    Some(Parts {
        uuid: &log_line[(left_paren + 1)..right_paren],
        timestamp: &log_line[(dash + 2)..right_bracket],
        message: &log_line[(right_bracket + 2)..],
    })
}


#[cfg(test)]
mod tests {
    use crate::lex::locate_parts;

    #[test]
    fn bad_format() {
        let log_line = b"not valid";
        let result = locate_parts(log_line);
        assert!(result.is_none());
    }

    #[test]
    fn old_format() {
        let log_line = b"Oct  1 20:43:45 shmeppy-0 shmeppy-app: (c4dfd175-c0ff-43d5-bd06-6366fd701030) [INFO - 10/1/2018 8:43:45 PM] Finished websocket: 1001 ''";
        let result = locate_parts(log_line);
        assert!(result.is_some());

        let parts = result.unwrap();
        assert_eq!(
            parts.uuid,
            &b"c4dfd175-c0ff-43d5-bd06-6366fd701030"[..]);
        assert_eq!(
            parts.timestamp,
            &b"10/1/2018 8:43:45 PM"[..]);
        assert_eq!(
            parts.message,
            &b"Finished websocket: 1001 ''"[..]);
    }

    #[test]
    fn new_format() {
        let log_line = b"shmeppy-1 shmeppy-app: (c722d1d6-86e6-4117-b444-e146b013859d) [INFO - 4/25/2020 9:55:14 PM] Committed 1 operation(s).";
        let result = locate_parts(log_line);
        assert!(result.is_some());

        let parts = result.unwrap();
        assert_eq!(
            parts.uuid,
            &b"c722d1d6-86e6-4117-b444-e146b013859d"[..]);
        assert_eq!(
            parts.timestamp,
            &b"4/25/2020 9:55:14 PM"[..]);
        assert_eq!(
            parts.message,
            &b"Committed 1 operation(s)."[..]);
    }
}
