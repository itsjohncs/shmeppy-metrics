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

    // * Transform the logs into requests with the information I need.
    //     * Simultaneously collect all the times when a game was modified and
    //       create a spet containing "the timespans the game had activity".
    // * Bucket the timespans of requests sharing (game_id, user_id) into spets
    //   (these are "user presences").
    // * Fold each bucket into a single spet containing all the times when N
    //   presences overlapped for a game.
    // * Intersect each of these spets with the times when the game had
    //   activity.
    // * Join any small gaps in each of these spets.
    // * Filter out any too-small timespans from these spets.
    // * Re-associate each spet with the information I need and then print
    //   them: the contiguous timespans in these final spets are the
    //   convocations I seek.
}
