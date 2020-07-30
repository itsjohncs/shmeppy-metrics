use std::io::stdin;
use std::io::BufRead;
use std::str::{FromStr, from_utf8};
use std::collections::BTreeMap;
use std::collections::HashSet;

use memchr::memchr;

mod extract_date;
use extract_date::extract_date;


fn extract_request_id(line: &[u8]) -> Option<[u8; 32]> {
    let start_id = memchr(b'(', line)? + 1;
    let end_id = start_id + memchr(b')', &line[start_id..])?;
    if end_id - start_id != 36 {
        return None;
    }

    // I wouldn't bother converting this to a smaller string, but Rust only
    // implements some important traits for arrays up to 32 bytes.
    let mut buffer = [b'!'; 32];

    // bbc823f4-f809-42cf-bcc5-8b3d5e6d9e9a -> bbc823f4f80942cfbcc58b3d5e6d9e9a
    buffer[..8].copy_from_slice(&line[start_id..][..8]);
    buffer[8..][..4].copy_from_slice(&line[start_id + 9..][..4]);
    buffer[12..][..4].copy_from_slice(&line[start_id + 14..][..4]);
    buffer[16..][..4].copy_from_slice(&line[start_id + 19..][..4]);
    buffer[20..][..12].copy_from_slice(&line[start_id + 24..][..12]);

    Some(buffer)
}


fn is_start_of_registration_request(line: &[u8]) -> bool {
    let right_bracket = match memchr(b']', line) {
        Some(n) => n,
        None => return false,
    };
    let msg = &line[right_bracket + 2..];

    let start = b"Started: POST /api/account/register/complete/";
    msg.len() >= start.len() && msg[..start.len()] == start[..]
}

fn maybe_get_request_status(line: &[u8]) -> Option<usize> {
    let right_bracket = memchr(b']', line)?;
    let msg = &line[right_bracket + 2..];

    let end = b"Finished: ";
    if msg[..end.len()] == end[..] {
        // Yes I intend to panic if any my assumptions about this line fails
        let space = end.len() + memchr(b' ', &msg[end.len()..]).unwrap();

        // Maybe use from_utf8_unchecked? Not sure how much overhead this'll
        // create... hopefully minimal.
        let code = usize::from_str(
            from_utf8(&msg[end.len()..space]).unwrap()).unwrap();
        Some(code)
    } else {
        None
    }
}


fn increment_registration_count(map: &mut BTreeMap<[u8; 10], usize>, k: &[u8; 10]) {
    match map.get_mut(k) {
        Some(count) => *count += 1,
        None => { map.insert(*k, 1); },
    };
}


fn main() {
    // Maps dates to number of registrations on that day
    let mut registration_counts: BTreeMap<[u8; 10], usize> = BTreeMap::new();

    // Stores any "complete registration" request IDs that we've seen
    let mut complete_registration_requests: HashSet<[u8; 32]> = HashSet::new();

    for maybe_line in stdin().lock().split(b'\n') {
        let line = maybe_line.unwrap();
        if let Some(id) = extract_request_id(&line) {
            if complete_registration_requests.contains(&id) {
                if let Some(code) = maybe_get_request_status(&line) {
                    complete_registration_requests.remove(&id);

                    if code == 200 {
                        increment_registration_count(
                            &mut registration_counts,
                            &extract_date(&line).unwrap());
                    }
                }
            } else if is_start_of_registration_request(&line) {
                complete_registration_requests.insert(id);
            }
        }
    }

    // Quick and dirty JSON serialization. Using serde is a bit of a pain
    // because of our byte string dates.
    print!("{{");
    let mut skipped_once = false;
    for (date, count) in registration_counts {
        if skipped_once {
            print!(",");
        } else {
            skipped_once = true;
        }
        print!("\"{}\":{}", std::str::from_utf8(&date).unwrap(), count);
    }
    println!("}}");
}
