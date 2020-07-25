mod lex;
use crate::lex::locate_parts;

mod parse;


fn main() {
	let ham = b"shmeppy-1 shmeppy-app: (c722d1d6-86e6-4117-b444-e146b013859d) [INFO - 4/25/2020 9:55:14 PM] Committed 1 operation(s).";
	let bob = locate_parts(ham);
    println!("ham {}", bob.unwrap());

    // Transform all logs into lists (separated by game ID) of
    // (timespan, user_identifier) tuples. (Vocab: each of these tuples is
    // called a "user presence").

    // Transform each of those lists into a single spet containing the timespans
    // where N number of users were present.

    // Transform that spet to join any small gaps

    // Transform that spet to remove too-short timespans

    // Print them
}
