use std::io::stdin;
use std::io::BufRead;
use std::collections::BTreeMap;

use serde::ser::{Serialize, Serializer};
use memchr::memchr;

mod extract_date;
use extract_date::extract_date;


fn get_message(line: &[u8]) -> Option<&[u8]> {
    let right_bracket = match memchr(b']', line) {
        Some(n) => n,
        None => return None,
    };
    
    Some(&line[right_bracket + 2..])
}


fn starts_with(msg: &[u8], prefix: &[u8]) -> bool {
    msg.len() >= prefix.len() && msg[..prefix.len()] == prefix[..]
}


fn increment(map: &mut BTreeMap<DateString, usize>, k: &DateString) {
    match map.get_mut(k) {
        Some(count) => *count += 1,
        None => { map.insert(*k, 1); },
    };
}


#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
struct DateString([u8; 10]);


impl Serialize for DateString {
    fn serialize<S: Serializer>(&self, serializer: S)
            -> Result<S::Ok, S::Error> {
        serializer.serialize_str(std::str::from_utf8(&self.0).unwrap())
    }
}


#[derive(Eq, PartialEq, Debug, Clone, Copy)]
struct Event {
    name: &'static str,
    prefix: &'static [u8]
}


fn main() {
    let events = [
        Event {
            name: "login",
            prefix: b"Started: POST /api/auth/login ",
        },  
        Event {
            name: "pend_registration",
            prefix: b"Started: POST /api/account/register ",
        },
        Event {
            name: "update_pended_registration",
            prefix: b"Started: PUT /api/account/register/",
        },
        Event {
            name: "reset_password_other",
            prefix: b"Started: POST /api/account/reset-password ",
        },
    ];

    // Using a nested map here, rather than something more efficient, so
    // that I don't have to write more serialization functions for serde...
    let mut counts: BTreeMap<&str, BTreeMap<DateString, usize>> = BTreeMap::new();
    for event in events.iter() {
        counts.insert(event.name, BTreeMap::new());
    }

    for maybe_line in stdin().lock().split(b'\n') {
        let line = maybe_line.unwrap();
        for event in events.iter() {
            if let Some(msg) = get_message(&line) {
                if starts_with(&msg, event.prefix) {
                    increment(counts.get_mut(event.name).unwrap(),
                              &DateString(extract_date(&line).unwrap()));
                }
            }
        }
    }

    println!("{}", serde_json::to_string(&counts).unwrap());
}
