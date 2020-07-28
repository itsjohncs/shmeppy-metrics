mod lex;
mod parse;
mod request;
mod activity;
mod presence;
mod accumulator;
mod timespan;

use std::io::BufRead;
use std::io::stdin;

use chrono::Duration;

use crate::request::RequestCollector;
use crate::lex::locate_parts;
use crate::activity::ActivityCollector;
use crate::presence::collect_presences;


fn main() {
    let mut activity_collector = ActivityCollector::new();
    let mut request_collector = RequestCollector::new();
    for maybe_line in stdin().lock().split(b'\n') {
        let line = maybe_line.unwrap();
        if let Some(parts) = locate_parts(&line) {
            request_collector.update(&parts);
            activity_collector.update(&parts);
        }
    }

    // I don't particularly like having the activity collector relying on the
    // request collector to get the game_id. But I also don't want to repeat
    // work in the hot-ass loop above... so this is an acceptable trade-off for
    // the performance I think.
    let game_id_to_activity = activity_collector.into_spets(
        |game_id| request_collector.game_id_for_request(game_id),
        Duration::minutes(30));

    let all_presences = collect_presences(request_collector.into_requests());

    // let convocations: Vec<Convocations> = Vec::new();
    // for (game_id, presences) in all_presences.iter() {
    //     // An iterator of timespans where a convocation was occurring
    //     let convocations_during =
    //         // We start with the set containing all the times N users were
    //         // present (check out the documentation for n_overlapping to see
    //         // how this translates to set operations if you like thinking about
    //         // all this in terms of sets like me).
    //         n_overlapping(2, presences.map(|presence| presence.spet))
    //             // Cut out any time users were present but nothing was
    //             // happening in the game. This prevents users who leave their
    //             // computers on all the time from messing with my analytics.
    //             .intersection(
    //                 activity_collector.active_during(game_id,
    //                                                  Duration::minutes(30)))
    //             // Join any "small" gaps between timespans. This is an attempt
    //             // to cut down on noise.
    //             .filter_complement(|start, end| match (start, end) {
    //                 (None, _) => true,
    //                 (_, None) => true,
    //                 (Some(a), Some(b)) => b - a > Duration::minutes(90),
    //             })
    //             // We now leave the land of spets and turn into an iterator
    //             // over timespans.
    //             .into_iter()
    //             // Filter out short timespans
    //             .filter(|span| span.end - span.start > Duration::minutes(40));

    //     // During these set operations we've lost the information of who is
    //     // participating in each convocation. Now we'll go and re-figure that
    //     // out.
    //     for timespan in convocations_during {
    //         let mut admins: BTreeSet<AccountId> = BTreeSet::new();
    //         let mut players: BTreeSet<AccountId> = BTreeSet::new();
    //         for presence in presences {
    //             if presence.spet.intersects_span(&timespan) {
    //                 if presence.is_admin {
    //                     admins.insert(presence.account_id);
    //                     players.remove(presence.account_id);
    //                 } else if !admins.contains(presence.account_id) {
    //                     players.insert(presence.account_id);
    //                 }
    //             }
    //         }

    //         if !admins.is_empty() {
    //             convocations.push(Convocation {
    //                 game_id,
    //                 during: timespan,
    //                 admins: Vec::from(admins),
    //                 players: Vec::from(players),
    //             });
    //         }
    //     }
    // }
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
