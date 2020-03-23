// Keep a (ring) queue of N lines (or bytes if lines ends up being impractical)
// Keep a state machine for each request:
//     start [bad version msg]-> bad_version [end msg]-> bad_version_end
//           [end msg]-> end
// When popping messages off the queue, print it unless bad_version_end is hit
// and mark the request ID as "being printed" (to avoid partially printing a
// request).

// Using bufreader directly seems like it oughta be fine. The fact that it's
// not quite a queue is a disadvantage though... I guess I could manually read
// into two buffers. I'd read the first 8 KB into buffer 1, then the second 8 KB
// into buffer 2, then the third 8 KB into buffer 1 again, alternating like
// that. That way I'll be able to look ahead 8 KB into the log each time to
// (hopefully) find the end of each request.

// This seems significantly simpler than the ring buffer idea, and keeps all
// the data very localized in memory.

// Doing this with a bunch of heap allocations using a ring buffer sounds super
// easy too though, and I could work on N lines that way and just do the state
// machine like I wanted. I wish I knew beforehand how expensive the heap
// allocations would end up being...

// Though now that I think about it, this isn't running on the whole log file,
// I'll be doing this as part of the "split everything up by day" pipeline, so
// it's ok for it to be expensive. Alright, I'll just do that then.

use std::collections::{VecDeque, HashSet};
use std::io::{stdin, stdout, Stdout, Write, BufRead};

use memchr::memchr;
use memmem::{Searcher, TwoWaySearcher};


// TODO: share this with the other script that uses this
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


fn is_bad_version_message(line: &[u8]) -> bool {
    let searcher = TwoWaySearcher::new(b"Bad version 0, expected 1.");
    searcher.search_in(line).is_some()
}


struct Filter {
    // Stores any requests we've printed one log message for (therefore we
    // should print any other log messages for the request).
    printed_request_ids: HashSet<[u8; 32]>,

    // Stores any requests we've seen a "bad version" message for (therefore
    // we shouldn't print any of its messages, unless we already printed
    // another message in the same request).
    bad_version_request_ids: HashSet<[u8; 32]>,

    // We gonna do a lot of heap allocations... One for each line. If it's too
    // slow we can try and use a smarter heap for this part, but I think we'll
    // be fine.
    queue: VecDeque<Vec<u8>>,
}

impl Filter {
    fn with_capacity(capacity: usize) -> Filter {
        Filter {
            printed_request_ids: HashSet::new(),
            bad_version_request_ids: HashSet::new(),
            queue: VecDeque::with_capacity(capacity),
        }
    }

    fn enqueue(self: &mut Filter, line: &[u8]) -> Option<Vec<u8>> {
        if is_bad_version_message(line) {
            if let Some(id) = extract_request_id(line) {
                self.bad_version_request_ids.insert(id);
            }
        }

        if self.queue.len() == self.queue.capacity() {
            let r = self.queue.pop_front().unwrap();
            self.queue.push_back(line.to_vec());
            Some(r)
        } else {
            self.queue.push_back(line.to_vec());
            None
        }
    }

    fn maybe_print(self: &mut Filter, out: &mut Stdout, line: &[u8]) {
        if let Some(id) = extract_request_id(&*line) {
            if self.printed_request_ids.contains(&id) ||
                    !self.bad_version_request_ids.contains(&id) {
                out.write(line).unwrap();
                out.write(b"\n").unwrap();
                self.printed_request_ids.insert(id);
            }
        } else {
            out.write(line).unwrap();
            out.write(b"\n").unwrap();
        }
    }

    fn flush(self: &mut Filter, out: &mut Stdout) {
        while let Some(line) = self.queue.pop_front() {
            self.maybe_print(out, &*line);
        }
    }
}


fn main() {
    let mut filter: Filter = Filter::with_capacity(1000);
    for maybe_line in stdin().lock().split(b'\n') {
        let line = maybe_line.unwrap();
        if let Some(popped_line) = filter.enqueue(&line) {
            filter.maybe_print(&mut stdout(), &popped_line);
        }
    }
    filter.flush(&mut stdout());
}
