use spet::mergeiter::sorted_chain;
use std::collections::BTreeMap;

use spet::vecspet::VecSpet;

use crate::request::{Request, UserId, GameId};
use crate::accumulator::push_onto_accumulator;
use crate::timespan::TimeSpan;


#[derive(Debug, Eq, PartialEq)]
pub struct GlobalPresence {
    pub spet: VecSpet<TimeSpan>,
    pub user_id: UserId,
}


pub fn collect_global_presences(
        requests: impl Iterator<Item=Request>,
        activity: BTreeMap<GameId, VecSpet<TimeSpan>>)
        -> Vec<GlobalPresence> {
    // Group each of a user's requests together
    let mut groups: BTreeMap<UserId, Vec<Request>> = BTreeMap::new();
    for request in requests {
        push_onto_accumulator(&mut groups, request.user_id, request);
    }

    groups.into_iter().map(|(user_id, requests)| {
        use spet::span::CreatableSpan;
        let spets: Vec<VecSpet<TimeSpan>> = 
            requests.iter()
                    .filter_map(|request| Some(activity.get(&request.game_id)?.intersection(&VecSpet::from_sorted_iter(vec![
                        TimeSpan::new(request.start, request.end)
                    ]))))
                    .collect();

        GlobalPresence {
            spet: VecSpet::from_sorted_iter(sorted_chain(spets)),
            user_id,
        }
    }).collect()
}


mod test {
    use std::collections::BTreeMap;

    use chrono::{Utc, TimeZone};
    use spet::vecspet::VecSpet;

    use crate::globalpresence::GlobalPresence;
    use crate::globalpresence::collect_global_presences;
    use crate::request::{Request, UserId, GameId};
    use crate::parse::UUID;
    use crate::timespan::TimeSpan;

    #[test]
    fn single_presence() {
        let requests = vec![
            Request {
                request_id: UUID(1),
                start: Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                end: Utc.ymd(2020, 1, 1).and_hms(1, 10, 0),
                game_id: 2,
                user_id: UserId::AccountId(3),
                is_admin: true,
            },
            Request {
                request_id: UUID(2),
                start: Utc.ymd(2020, 1, 1).and_hms(1, 11, 0),
                end: Utc.ymd(2020, 1, 1).and_hms(1, 20, 0),
                game_id: 2,
                user_id: UserId::AccountId(3),
                is_admin: true,
            },
        ];

        let mut activity: BTreeMap<GameId, VecSpet<TimeSpan>> = BTreeMap::new();
        activity.insert(2, VecSpet::from_sorted_iter(vec![
            TimeSpan::new(
                Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                Utc.ymd(2020, 1, 1).and_hms(1, 19, 0))
        ]));


        let presences = collect_global_presences(requests.into_iter(), activity);
        assert_eq!(presences.len(), 1);

        use spet::span::CreatableSpan;
        assert_eq!(presences, vec![
            GlobalPresence {
                user_id: UserId::AccountId(3),
                spet: VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 10, 0)),
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 11, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 19, 0)),
                ]),
            }
        ]);
    }

    #[test]
    fn multiple_presences() {
        let requests = vec![
            Request {
                request_id: UUID(1),
                start: Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                end: Utc.ymd(2020, 1, 1).and_hms(1, 10, 0),
                game_id: 2,
                user_id: UserId::AccountId(3),
                is_admin: true,
            },
            Request {
                request_id: UUID(2),
                start: Utc.ymd(2020, 1, 1).and_hms(1, 11, 0),
                end: Utc.ymd(2020, 1, 1).and_hms(1, 20, 0),
                game_id: 2,
                user_id: UserId::AccountId(4),
                is_admin: false,
            },
        ];

        let mut activity: BTreeMap<GameId, VecSpet<TimeSpan>> = BTreeMap::new();
        activity.insert(2, VecSpet::from_sorted_iter(vec![
            TimeSpan::new(
                Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                Utc.ymd(2020, 1, 1).and_hms(1, 19, 0))
        ]));

        let presences = collect_global_presences(requests.into_iter(),
                                                 activity);

        use spet::span::CreatableSpan;
        assert_eq!(presences, vec![
            GlobalPresence {
                user_id: UserId::AccountId(3),
                spet: VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 10, 0)),
                ]),
            },
            GlobalPresence {
                user_id: UserId::AccountId(4),
                spet: VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 11, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 19, 0)),
                ]),
            },
        ]);
    }
}
