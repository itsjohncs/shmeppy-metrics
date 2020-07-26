mod lex;
mod parse;
mod request;

use std::io::BufRead;
use std::io::stdin;

use crate::request::RequestCollector;
use crate::lex::locate_parts;


fn main() {
    let mut collector = RequestCollector::new();
    for maybe_line in stdin().lock().split(b'\n') {
        let line = maybe_line.unwrap();
        if let Some(parts) = locate_parts(&line) {
            collector.update(&parts);
        }
    }

    let requests: Vec<_> = collector.into_requests().collect();

    println!("Num requests: {}", requests.len());

    // Transform all logs into lists (separated by game ID) of
    // (timespan, user_identifier) tuples. (Vocab: each of these tuples is
    // called a "user presence").

    // Transform each of those lists into a single spet containing the timespans
    // where N number of users were present.

    // Transform that spet to join any small gaps

    // Transform that spet to remove too-short timespans

    // Print them
}
