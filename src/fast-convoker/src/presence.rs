use std::collections::BTreeMap;

use spet::vecspet::VecSpet;

use crate::request::{Request, GameId, UserId};
use crate::accumulator::push_onto_accumulator;
use crate::timespan::TimeSpan;


#[derive(Debug, Eq, PartialEq)]
pub struct Presence {
    pub spet: VecSpet<TimeSpan>,
    pub game_id: GameId,
    pub user_id: UserId,
    pub is_admin: bool,
}


fn get_key(request: &Request) -> (GameId, UserId, bool) {
    (request.game_id, request.user_id, request.is_admin)
}


pub fn collect_presences(requests: impl Iterator<Item=Request>)
        -> BTreeMap<GameId, Vec<Presence>> {
    // We'll collect all the requests that belong in a single request into
    // buckets.
    let mut groups: BTreeMap<(GameId, UserId, bool), Vec<Request>> =
        BTreeMap::new();
    for request in requests {
        push_onto_accumulator(&mut groups, get_key(&request), request);
    }

    // Now we'll actually create the presences...
    let all_presences =
        groups.into_iter().map(|((game_id, user_id, is_admin), requests)| {
            use spet::span::CreatableSpan;
            let mut spans: Vec<TimeSpan> = 
                requests.iter()
                        .map(|request| TimeSpan::new(request.start,
                                                     request.end))
                        .collect();
            spans.sort_unstable();

            Presence {
                spet: VecSpet::from_sorted_iter(spans.into_iter()),
                game_id,
                user_id,
                is_admin
            }
        });

    // and group those presences by game_id
    let mut grouped_presences = BTreeMap::new();
    for presence in all_presences {
        push_onto_accumulator(&mut grouped_presences, presence.game_id,
                              presence);
    }

    grouped_presences
}


mod test {
    use chrono::{Utc, TimeZone};
    use spet::vecspet::VecSpet;

    use crate::presence::Presence;
    use crate::presence::collect_presences;
    use crate::request::{Request, UserId};
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

        let presences_by_game = collect_presences(requests.into_iter());
        assert_eq!(presences_by_game.len(), 1);

        let (_, presences) = presences_by_game.iter().next().unwrap();
        use spet::span::CreatableSpan;
        assert_eq!(presences, &vec![
            Presence {
                game_id: 2,
                user_id: UserId::AccountId(3),
                is_admin: true,
                spet: VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 10, 0)),
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 11, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 20, 0)),
                ]),
            }
        ]);
    }

    #[test]
    fn is_admin_changed() {
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
                is_admin: false,
            },
        ];

        let presences_by_game = collect_presences(requests.into_iter());
        assert_eq!(presences_by_game.len(), 1);

        let (_, presences) = presences_by_game.iter().next().unwrap();
        use spet::span::CreatableSpan;
        assert_eq!(presences, &vec![
            Presence {
                game_id: 2,
                user_id: UserId::AccountId(3),
                is_admin: false,
                spet: VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 11, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 20, 0)),
                ]),
            },
            Presence {
                game_id: 2,
                user_id: UserId::AccountId(3),
                is_admin: true,
                spet: VecSpet::from_sorted_iter(vec![
                    TimeSpan::new(
                        Utc.ymd(2020, 1, 1).and_hms(1, 0, 0),
                        Utc.ymd(2020, 1, 1).and_hms(1, 10, 0)),
                ]),
            },
        ]);
    }
}
