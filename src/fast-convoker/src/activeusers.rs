mod lex;
mod parse;
mod request;
mod activity;
mod accumulator;
mod timespan;
mod globalpresence;


use std::io::stdout;
use std::collections::BTreeMap;
use std::io::BufRead;
use std::io::{stdin};

use serde::ser::{Serialize, Serializer};
use chrono::{Duration, Date, Utc, Datelike};
use chrono::offset::TimeZone;
use spet::span::CreatableSpan;
use spet::span::Span;
use spet::vecspet::VecSpet;

use crate::timespan::TimeSpan;
use crate::request::{RequestCollector, UserId};
use crate::lex::locate_parts;
use crate::activity::ActivityCollector;
use crate::globalpresence::collect_global_presences;


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


fn total_time(spet: &VecSpet<TimeSpan>) -> Duration {
    spet.into_iter().fold(Duration::zero(), |a, i| a + (*i.end() - *i.start()))
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

    let activity = activity_collector.into_spets(
        |game_id| request_collector.game_id_for_request(game_id),
        Duration::minutes(30));

    // Unlike in the fast-convoker, we take into account the activity data
    // when we generate our presences. This is because, unlike in
    // fast-convoker, we're not keying presences on game ID.
    let global_presences = collect_global_presences(
        request_collector.into_requests().filter(|r| r.is_admin),
        activity);

    // {day: {user_id: active_seconds}}
    let mut result: BTreeMap<String, BTreeMap<UserId, i64>> = BTreeMap::new();

    let today = Utc::now().date();
    let mut current_day = Utc.ymd(2018, 1, 1);
    while current_day <= today {
        let day_spet = VecSpet::<TimeSpan>::from_sorted_iter(vec![
            TimeSpan::new(
                current_day.and_hms(0, 0, 0),
                current_day.succ().and_hms(0, 0, 0) - Duration::nanoseconds(1)),
        ]);
        let day = format!("{}-{}-{}", current_day.year(), current_day.month(),
                          current_day.day());
        result.insert(day.clone(), BTreeMap::new());

        let user_to_seconds = result.get_mut(&day).unwrap();
        for presence in &global_presences {
            let seconds = total_time(&presence.spet.intersection(&day_spet)).num_seconds();
            if seconds > 0 {
                user_to_seconds.insert(presence.user_id, seconds);
            }
        }

        current_day = current_day.succ();
    }

    serde_json::ser::to_writer(stdout(), &result).unwrap();
}
