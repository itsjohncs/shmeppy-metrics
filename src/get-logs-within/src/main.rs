/**
 * This program is designed to churn through all of the log files Shmeppy
 * has ever made on-demand and produce only the logs that fall within a
 * particular range.
 *
 * It needs to run very quickly, as it's used by `build-convocations-aggregate`
 * to grab all the logs in a three-day span, for every day since Shmeppy's
 * inception. Fortunately `build-convocations-aggregate` does some smart
 * caching across runs so it doesn't actually repeat itself more than
 * necessary, but this is still going to be run quite often.
 */

use std::process::exit;
use std::collections::HashSet;
use std::env::Args;
use std::io::Write;
use memchr::memchr;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdout};
use std::iter::Iterator;


fn number_or_zero(b: u8) -> u8 {
    if b'0' <= b && b <= b'9' {
        b
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
 * It also rearranges the date to match YYYY-MM-DD. This is unnecessary for its
 * current use in this script, but I originally wrote it for a different
 * purpose.
 */
fn extract_date(line: &[u8]) -> Option<[u8; 10]> {
    let left_bracket = memchr(b'[', line)?;
    let first_slash = left_bracket + memchr(b'/', &line[left_bracket..])?;
    let second_slash = 2 + first_slash + memchr(b'/', &line[first_slash + 1..])?;

    let mut buffer: [u8; 10] = [b'-'; 10];
    buffer[5] = number_or_zero(line[first_slash - 2]);
    buffer[6] = line[first_slash - 1];
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
    buffer[0..4].copy_from_slice(&line[second_slash..second_slash + 4]);

    Some(buffer)
}


fn parse_date_arg(date: &str) -> Option<[u8; 10]> {
	let bytes = date.as_bytes();

	// As much validation as anyone's getting from this script 😂
	if date.len() == 10 {
		let mut buf = [0; 10];
		buf.copy_from_slice(bytes);
		Some(buf)
	} else {
		None
	}
}


fn parse_args(mut args: Args) -> Option<(HashSet<[u8; 10]>, Vec<String>)> {
	let mut dates: HashSet<[u8; 10]> = HashSet::new();

	for date in args.by_ref().skip(1).take_while(|arg| arg != "--") {
		dates.insert(parse_date_arg(&date.as_str())?);
	}

	let file_paths: Vec<String> = args.collect();

	if dates.len() == 0 || file_paths.len() == 0 {
		None
	} else {
		Some((dates, file_paths))
	}
}


fn main() {
	let (dates, file_paths) = match parse_args(env::args()) {
		Some(x) => x,
		None => {
			eprintln!("{} YYYY-MM-DD [YYYY-MM-DD ...] -- FILE [FILE ...]",
				 	  env::args().next().unwrap());
			exit(1);
		},
	};

    let mut stdout = stdout();
    for file_path in file_paths {
        let file = File::open(&file_path).expect(
            &format!("Can't open {}", file_path));
        for maybe_line in BufReader::new(file).split(b'\n') {
        	let line = maybe_line.unwrap();
            if let Some(date) = extract_date(&line) {
            	if dates.contains(&date) {
            		stdout.write(&line).unwrap();
            		stdout.write(b"\n").unwrap();
            	}
            }
        }
    }
}
