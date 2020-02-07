/**
 * This program will churn through all of the Shmeppy logs given to it through
 * its stdin and then print two dates in the format YYYY-MM-DD separated by a
 * single space. The first date is the earliest date it saw in a log timestamp,
 * and the second is the latest date it saw in a log timestamp.
 */

use std::io::{BufRead, stdin};

mod extract_date;


fn main() {
    let mut earliest = None;
    let mut latest = None;

    for maybe_line in stdin().lock().split(b'\n') {
        let line = maybe_line.unwrap();
        if let Some(date) = extract_date::extract_date(&line) {
            match earliest {
                Some(old) if date < old => earliest = Some(date),
                None => earliest = Some(date),
                _ => (),
            };

            match latest {
                Some(old) if date > old => latest = Some(date),
                None => latest = Some(date),
                _ => (),
            };
        }
    }

    if let (Some(a), Some(b)) = (earliest, latest) {
        println!("{} {}",
            std::str::from_utf8(&a).unwrap(),
            std::str::from_utf8(&b).unwrap());
    }
}
