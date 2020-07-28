use std::collections::BTreeMap;

use chrono::{DateTime, Utc, Duration};
use spet::vecspet::VecSpet;

use crate::request::GameId;
use crate::lex::Parts;
use crate::parse::{UUID, parse_uuid, parse_timestamp};
use crate::accumulator::push_onto_accumulator;
use crate::timespan::TimeSpan;


pub struct ActivityCollector {
    active_times: BTreeMap<UUID, Vec<DateTime<Utc>>>,
}


fn is_activity_message(msg: &[u8]) -> bool {
    let expected_prefix = b"Committed ";
    let expected_suffix = b" operation(s).";
    msg[..expected_prefix.len()] == *expected_prefix &&
        msg[msg.len() - expected_suffix.len()..] == *expected_suffix
}


impl ActivityCollector {
    pub fn new() -> ActivityCollector {
        ActivityCollector { active_times: BTreeMap::new() }
    }

    pub fn update(&mut self, parts: &Parts) -> Option<()> {
        if is_activity_message(parts.message) {
            let uuid = parse_uuid(parts.uuid)?;
            let timestamp = parse_timestamp(parts.timestamp)?;
            push_onto_accumulator(&mut self.active_times, uuid, timestamp);

            Some(())
        } else {
            None
        }
    }

    pub fn into_spets(
            self,
            game_id_for_request: impl Fn(UUID) -> Option<GameId>,
            window_size: Duration) -> BTreeMap<GameId, VecSpet<TimeSpan>> {
        let mut by_game_id: BTreeMap<GameId, Vec<DateTime<Utc>>> = BTreeMap::new();
        for (uuid, timestamps) in self.active_times {
            if let Some(game_id) = game_id_for_request(uuid) {
                for timestamp in timestamps {
                    push_onto_accumulator(&mut by_game_id, game_id, timestamp);
                }
            }
        }

        let mut result = BTreeMap::new();
        for (game_id, mut timestamps) in by_game_id {
            use spet::span::CreatableSpan;
            timestamps.sort_unstable();
            let spans = timestamps.into_iter().map(|timestamp|
                TimeSpan::new(
                    timestamp - window_size / 2,
                    timestamp + window_size / 2));

            result.insert(game_id, VecSpet::from_sorted_iter(spans));
        }

        result
    }
}


#[cfg(test)]
mod tests {
    use spet::vecspet::VecSpet;
    use spet::span::CreatableSpan;
    use chrono::offset::TimeZone;
    use chrono::{Utc, Duration};

    use crate::parse::UUID;
    use crate::activity::{ActivityCollector, TimeSpan};
    use crate::lex::locate_parts;

    #[test]
    fn standard_use() {
        let lines = vec![
            b"shmeppy-1 shmeppy-app: (d2deee0c-9fd8-446c-9506-be65bbac5206) [INFO - 5/26/2020 3:33:19 PM] Committed 1 operation(s).",
            b"shmeppy-1 shmeppy-app: (ce3f74d8-5e3c-48de-8411-d0663861bed8) [INFO - 5/26/2020 3:33:21 PM] Committed 1 operation(s).",
            b"shmeppy-1 shmeppy-app: (ce3f74d8-5e3c-48de-8411-d0663861bed8) [INFO - 5/26/2020 4:33:22 PM] Committed 1 operation(s).",
        ];

        let mut collector = ActivityCollector::new();
        for line in lines {
            collector.update(&locate_parts(line).unwrap());
        }

        assert_eq!(
            collector.active_times.clone().into_iter().collect::<Vec<_>>(),
            vec![
                (UUID(0xce3f74d8_5e3c_48de_8411_d0663861bed8), vec![
                    Utc.ymd(2020, 5, 26).and_hms(15, 33, 21),
                    Utc.ymd(2020, 5, 26).and_hms(16, 33, 22),
                ]),
                (UUID(0xd2deee0c_9fd8_446c_9506_be65bbac5206), vec![
                    Utc.ymd(2020, 5, 26).and_hms(15, 33, 19),
                ]),
            ]);

        let spets = collector.into_spets(
            |game_id| match game_id {
                UUID(0xce3f74d8_5e3c_48de_8411_d0663861bed8) => Some(1),
                UUID(0xd2deee0c_9fd8_446c_9506_be65bbac5206) => Some(2),
                _ => None,
            },
            Duration::minutes(10));
        assert_eq!(
            spets.into_iter().collect::<Vec<_>>(),
            vec![
                (1, VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 5, 26).and_hms(15, 28, 21),
                        Utc.ymd(2020, 5, 26).and_hms(15, 38, 21)),
                    TimeSpan::new(
                        Utc.ymd(2020, 5, 26).and_hms(16, 28, 22),
                        Utc.ymd(2020, 5, 26).and_hms(16, 38, 22)),
                ])),
                (2, VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 5, 26).and_hms(15, 28, 19),
                        Utc.ymd(2020, 5, 26).and_hms(15, 38, 19))
                ])),
            ])
    }
}
