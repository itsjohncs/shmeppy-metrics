use memchr::memchr;

fn number_or_zero(byte: u8) -> u8 {
    if b'0' <= byte && byte <= b'9' {
        byte
    } else {
        b'0'
    }
}


/**
 * Extracts a date from the log line.
 *
 * This formats the date such that its month and day is zero-padded (it's not
 * in the logs).
 *
 * It also rearranges the date to match YYYY-MM-DD (ie: the date can be sorted
 * both by an application that understands the actual date or by an application
 * that just treats it as a string).
 *
 * This needs to be able to run millions of times per second, and is a hot
 * path.
 */
pub fn extract_date(line: &[u8]) -> Option<[u8; 10]> {
    // Find our three pillars that we'll orient ourselves around. We're looking
    // for something like 1/20/2019.
    let left_bracket = memchr(b'[', line)?;
    let first_slash = left_bracket + memchr(b'/', &line[left_bracket..])?;
    let second_slash = 2 + first_slash + memchr(b'/', &line[first_slash + 1..])?;

    let mut buffer: [u8; 10] = [b'-'; 10];

    // Month goes from the start of the string to the middle
    buffer[5] = number_or_zero(line[first_slash - 2]);
    buffer[6] = line[first_slash - 1];

    // Day goes from the middle to the end
    match second_slash - first_slash {
        3 => {
            buffer[8] = b'0';
            buffer[9] = line[first_slash + 1];
        },
        4 => {
            buffer[8] = line[first_slash + 1];
            buffer[9] = line[first_slash + 2];
        },
        _ => return None
    }

    // Year goes from the end to the beginning
    buffer[0..4].copy_from_slice(&line[second_slash..second_slash + 4]);

    Some(buffer)
}
