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
 *
 * Currently it trails `wc -l` by just a little bit. Because of the filesystem
 * caching that's bound to happen, memory maps should speed this up. But I
 * think if this starts getting slow a simple short-circuit will help: peek
 * the beginning and end of each file to get the range of days in each file
 * (give or take a day) and skip it if it's out of range.
 */

use std::process::exit;
use std::collections::HashSet;
use std::env::Args;
use std::io::Write;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdout};
use std::iter::Iterator;

mod extract_date;


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
    // A vector should perform better than this hash set, but it's not a
    // bottleneck so eh.
    let mut dates: HashSet<[u8; 10]> = HashSet::new();

    // We're going to consume the args iterator bit-by-bit starting with the
    // dates here.
    for date in args.by_ref().skip(1).take_while(|arg| arg != "--") {
        dates.insert(parse_date_arg(&date.as_str())?);
    }

    // The remaining args are the file paths (-- gets consumed by take_while)
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

        // I'm careful to never decode the file, that increases the run time
        // by about 5x.
        for maybe_line in BufReader::new(file).split(b'\n') {
            let line = maybe_line.unwrap();
            if let Some(date) = extract_date::extract_date(&line) {
                if dates.contains(&date) {
                    stdout.write(&line).unwrap();
                    stdout.write(b"\n").unwrap();
                }
            }
        }
    }
}
