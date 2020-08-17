mod lex;
mod parse;
mod request;
mod activity;
mod presence;
mod accumulator;
mod timespan;

use std::collections::{BTreeSet, BTreeMap};
use std::io::BufRead;
use std::io::{stdin, stdout};

use chrono::Duration;
use spet::span::Span;
use spet::vecspet::VecSpet;
use spet::overlapping::n_overlapping;
use serde::ser::{Serialize, Serializer, SerializeMap};

use crate::timespan::TimeSpan;
use crate::request::{RequestCollector, GameId, UserId};
use crate::lex::locate_parts;
use crate::activity::ActivityCollector;
use crate::presence::collect_presences;
use crate::accumulator::push_onto_accumulator;


#[derive(Debug)]
struct Convocation {
    game_id: GameId,
    during: TimeSpan,
    admins: Vec<UserId>,
    players: Vec<UserId>,
}


impl Serialize for Convocation {
    fn serialize<S: Serializer>(&self, serializer: S)
            -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("game_id", &self.game_id)?;
        map.serialize_entry("start", &self.during.start().to_rfc3339())?;
        map.serialize_entry("end", &self.during.end().to_rfc3339())?;
        map.serialize_entry("admins", &self.admins)?;
        map.serialize_entry("players", &self.players)?;
        map.end()
    }
}


impl Serialize for UserId {
    fn serialize<S: Serializer>(&self, serializer: S)
            -> Result<S::Ok, S::Error> {
        use UserId::*;
        use crate::parse::UUID;
        serializer.serialize_str(match self {
            AnalyticsId(UUID(uuid)) => format!("analytics_id:{}", uuid),
            AccountId(id) => format!("account_id:{}", id),
            Anonymous => format!("anonymous"),
        }.as_str())
    }
}


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

    let mut convocations_by_day: BTreeMap<String, Vec<Convocation>> =
            BTreeMap::new();
    for (game_id, presences) in all_presences.iter() {
        // An iterator of timespans where a convocation was occurring
        let convocations_during: VecSpet<TimeSpan> =
            VecSpet::from_sorted_iter(
                // We start with the set containing all the times N users were
                // present.
                n_overlapping(3, presences.iter().map(|presence| &presence.spet))
                    // Cut out any time users were present but nothing was
                    // happening in the game. This prevents users who leave their
                    // computers on all the time from messing with my analytics.
                    // Note: this is the same as doing it to each presence's
                    // spet beforehand, because n_overlapping is equivalent to
                    // (A & B) | (A & C) | (B & C) (for n = 2 over 3 elements,
                    // but hopefully the pattern is clear). So intersecting the
                    // result of that is the same as intersecting each
                    // individual product (distributive laws of set operations
                    // loosely match those of multiplication and addition
                    // here).
                    .intersection(
                        game_id_to_activity.get(game_id)
                                           .unwrap_or(&VecSpet::default()))
                    // Now close any small gaps. We'll do another filtering of
                    // gaps at the end so we don't end up with duplicate
                    // convocations (like if a group takes a break for a bit
                    // and then returns). But this is to cover mundane things
                    // like spontaneous disconnections and such.
                    .filter_gaps(|start, end| *end - *start < Duration::minutes(5))
                    // Filter out short convocations
                    .into_iter()
                    .filter(|span| *span.end() - *span.start() > Duration::minutes(40))
            ).filter_gaps(|start, end| *end - *start < Duration::minutes(90));

        // During these set operations we've lost the information of who is
        // participating in each convocation. Now we'll go and re-figure that
        // out.
        for timespan in convocations_during {
            let mut admins: BTreeSet<UserId> = BTreeSet::new();
            let mut players: BTreeSet<UserId> = BTreeSet::new();
            for presence in presences {
                let spet_span = VecSpet::from_sorted_iter(vec![timespan]);
                if !presence.spet.intersection(&spet_span).is_empty() {
                    if presence.is_admin {
                        admins.insert(presence.user_id);
                        players.remove(&presence.user_id);
                    } else if !admins.contains(&presence.user_id) {
                        players.insert(presence.user_id);
                    }
                }
            }

            if !admins.is_empty() {
                push_onto_accumulator(
                    &mut convocations_by_day,
                    timespan.start().format("%Y-%m-%d").to_string(),
                    Convocation {
                        game_id: *game_id,
                        during: timespan,
                        admins: admins.into_iter().collect(),
                        players: players.into_iter().collect(),
                    });
            }
        }
    }

    serde_json::ser::to_writer(stdout(), &convocations_by_day).unwrap();
}
