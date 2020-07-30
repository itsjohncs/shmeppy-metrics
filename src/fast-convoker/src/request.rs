use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use memmem::{Searcher, TwoWaySearcher};

use crate::lex::Parts;
use crate::parse::{parse_timestamp, parse_uuid};
use crate::parse::UUID;


pub type GameId = u64;
pub type AccountId = u64;


#[derive(PartialEq, Eq, Debug)]
enum Message {
    // Contains game ID
    StartedGameId(GameId),

    // Contains analytics ID
    AnalyticsId(UUID),

    // Contains account_id and is_admin
    AuthenticatedAs(AccountId, bool),

    Finished,
}


fn skip_prefix<'a>(prefix: &[u8], from: &'a [u8]) -> Option<&'a [u8]> {
    if from.len() >= prefix.len() && prefix == &from[..prefix.len()] {
        Some(&from[prefix.len()..])
    } else {
        None
    }
}


// Extracts a base 10 ascii-encoded integer from the very start of raw. So
// b"12 hello" would get 12 extracted, but b" 12 hello" would give None.
fn extract_integer<T: std::str::FromStr>(raw: &[u8]) -> Option<T> {
    let mut i = 0;
    while i < raw.len() {
        if raw[i] < b'0' || raw[i] > b'9' {
            break;
        }

        i += 1;
    }

    let integer_as_str = unsafe {
        std::str::from_utf8_unchecked(&raw[..i])
    };

    integer_as_str.parse::<T>().ok()
}


fn extract_account_id_from_old_log(raw: &[u8]) -> Option<AccountId> {
    fn is_ascii_hex_character(c: u8) -> bool {
        (b'0' <= c && c <= b'9') || (b'a' <= c && c <= b'f')
    }

    let looking_for = b"{ accountId: '";
    let start = TwoWaySearcher::new(looking_for).search_in(raw)? +
                looking_for.len();
    let mut end = start;
    while end < raw.len() && is_ascii_hex_character(raw[end]) {
        end += 1;
    }

    let account_id_str = unsafe {
        std::str::from_utf8_unchecked(&raw[start..end])
    };

    AccountId::from_str_radix(account_id_str, 16).ok()
}


fn extract_is_admin_from_old_log(raw: &[u8]) -> Option<bool> {
    let looking_for = b"#012  isAdmin: ";
    let start = TwoWaySearcher::new(looking_for).search_in(raw)? +
                looking_for.len();
    let cut_raw = &raw[start..];

    if skip_prefix(b"true", cut_raw).is_some() {
        Some(true)
    } else if skip_prefix(b"false", cut_raw).is_some() {
        Some(false)
    } else {
        None
    }
}


fn parse_message(raw: &[u8]) -> Option<Message> {
    use Message::*;

    if let Some(remainder) = skip_prefix(
            b"Started websocket: GET /game-socket/", raw) {
        Some(StartedGameId(extract_integer(remainder)?))
    } else if let Some(remainder) = skip_prefix(
            b"Analytics ID: ", raw) {
        Some(AnalyticsId(crate::parse::parse_uuid(remainder)?))
    } else if let Some(remainder) = skip_prefix(
            b"Client added to client DB: ", raw) {
        // This detects whether this is a log message from before I changed the
        // log to be in JSON format.
        if skip_prefix(b"{ gameId", remainder).is_some() {
            Some(AuthenticatedAs(
                extract_account_id_from_old_log(raw)?,
                extract_is_admin_from_old_log(raw)?))
        } else {
            use serde_json::Value;
            
            let json: Value = serde_json::from_slice(remainder).ok()?;
            if let (Value::String(raw_account_id), Value::Bool(is_admin)) =
                    (json.get("account")?.get("accountId")?,
                     json.get("isAdmin")?) {
                Some(AuthenticatedAs(
                    AccountId::from_str_radix(raw_account_id.as_str(), 16).ok()?,
                    *is_admin))
            } else {
                None
            }
        }
    } else if skip_prefix(b"Finished websocket: ", raw).is_some() ||
            skip_prefix(b"Finished: ", raw).is_some() ||
            skip_prefix(b"Waiting for client to complete closing handshake.", raw).is_some() {
        Some(Finished)
    } else {
        None
    }
}


#[derive(PartialEq, Eq, Debug, Clone, Copy, Ord, PartialOrd)]
pub enum UserId {
    AnalyticsId(UUID),
    AccountId(AccountId),
    Anonymous,
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Request {
    pub request_id: UUID,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub game_id: GameId,
    pub user_id: UserId,
    pub is_admin: bool,
}


impl Request {
    fn from_partial(request_id: UUID, partial: &PartialRequest) -> Option<Request> {
        Some(Request {
            request_id,
            start: partial.start?,
            end: partial.end?,
            game_id: partial.game_id?,
            is_admin: partial.is_admin?,
            user_id: match (partial.account_id, partial.analytics_id) {
                (None, Some(analytics_id)) =>
                    UserId::AnalyticsId(analytics_id),
                (Some(account_id), _) => UserId::AccountId(account_id),
                _ => UserId::Anonymous
            },
        })
    }
}


#[derive(Default)]
struct PartialRequest {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    game_id: Option<GameId>,
    analytics_id: Option<UUID>,
    account_id: Option<AccountId>,
    is_admin: Option<bool>,
    // The UUID of the request is associated with this in the hash table, so
    // it's not included here.
}


pub struct RequestCollector {
    partial_requests: BTreeMap<UUID, PartialRequest>
}


impl RequestCollector {
    pub fn new() -> RequestCollector {
        RequestCollector {
            partial_requests: BTreeMap::new(),
        }
    }

    pub fn update(&mut self, parts: &Parts) -> Option<()> {
        use Message::*;
        let message = parse_message(parts.message)?;

        // Grabs the request object for this request, or makes it if this is
        // a "StartedGameId" message.
        let partial_request: &mut PartialRequest = {
            let uuid = parse_uuid(parts.uuid)?;
            if let StartedGameId(_) = message {
                self.partial_requests.insert(uuid,
                                             PartialRequest::default());
            }

            self.partial_requests.get_mut(&uuid)?
        };

        match message {
            StartedGameId(game_id) => {
                partial_request.game_id = Some(game_id);
                partial_request.start = parse_timestamp(parts.timestamp);
            },
            AnalyticsId(analytics_id) => {
                partial_request.analytics_id = Some(analytics_id);
            },
            AuthenticatedAs(account_id, is_admin) => {
                partial_request.account_id = Some(account_id);
                partial_request.is_admin = Some(is_admin);
            }
            Finished => {
                // We look for a few messages as the "end" of a request, and
                // it's common to have them appear more than once. So we wanna
                // make sure to just call the request ended once we hit the
                // first one.
                if partial_request.end.is_none() {
                    partial_request.end = parse_timestamp(parts.timestamp);
                }
            },
        }

        Some(())
    }

    pub fn into_requests(self) -> impl Iterator<Item = Request> {
        self.partial_requests
            .into_iter()
            .filter_map(|(uuid, partial)| Request::from_partial(uuid, &partial))
    }

    pub fn game_id_for_request(&self, request_id: UUID) -> Option<GameId> {
        self.partial_requests.get(&request_id)?.game_id
    }
}


#[cfg(test)]
mod tests {
    mod request_collector {
        use crate::request::{RequestCollector, Request, UserId};
        use crate::lex::locate_parts;
        use crate::parse::UUID;
        use chrono::Utc;
        use chrono::offset::TimeZone;

        #[test]
        fn complete_and_partial() {
            let raw_logs: Vec<&[u8]> = vec![
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:33:47 AM] Started websocket: GET /game-socket/381468491?version=2&lastSeenIndex=328 1.1"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:33:47 AM] Analytics ID: 45e0e69c-ddd9-4443-abab-f3b46e47a62b"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:35:08 AM] Successfully authenticated as: { accountId: 'c7d585cd803aafa5', displayName: 'John' }"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:35:08 AM] Joining game: { adminAccountId: 'c7d585cd803aafa5', gameId: 381468491 }"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:35:08 AM] Client added to client DB: {"gameId":388491,"clientId":18378,"account":{"accountId":"c7d585cd803aafa5","displayName":"John"},"isAdmin":true,"initialLastSeenIndex":328,"lastPing":1590456908483}"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:35:08 AM] timing mark {"context":"operationsService","time":[10021127,60426389],"starting":["get operations after"],"ending":[]}"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:35:08 AM] timing mark {"context":"operationsService","time":[10021127,62358039],"starting":["sending data (update some with ops)"],"ending":[]}"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:35:08 AM] timing mark {"context":"operationsService","time":[10021127,63065222],"starting":[],"ending":["get operations after","sending data (update some with ops)"]}"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:35:08 AM] Socket init complete"#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:23 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:25 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:27 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:29 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:31 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:33 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:35 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:37 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:40 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:42 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:44 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:47 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [DEBUG - 5/26/2020 1:36:49 AM] Waiting for client to complete closing handshake."#,
                br#"shmeppy-1 shmeppy-app: (357edc79-03fe-4787-910a-d2f14302348c) [INFO - 5/26/2020 1:36:50 AM] Finished websocket: 1005 ''"#,
            ];

            let mut collector = RequestCollector::new();
            for raw_log in raw_logs {
                collector.update(&locate_parts(raw_log).unwrap());
            }

            let partial_requests: Vec<_> =
                collector.partial_requests.iter().collect();
            assert_eq!(partial_requests.len(), 1);

            let (uuid, partial_request) = partial_requests[0];
            let maybe_request = Request::from_partial(*uuid, partial_request);
            let expected_request = Request {
                request_id: UUID(0x357edc79_03fe_4787_910a_d2f14302348c),
                start: Utc.ymd(2020, 5, 26).and_hms(1, 33, 47),
                end: Utc.ymd(2020, 5, 26).and_hms(1, 36, 23),
                game_id: 381468491,
                user_id: UserId::AccountId(0xc7d585cd803aafa5),
                is_admin: true,
            };
            assert_eq!(maybe_request, Some(expected_request.clone()));

            assert_eq!(collector.into_requests().collect::<Vec<_>>(),
                       vec![expected_request]);
        }
    }

    mod extract_is_admin_from_old_log {
        use crate::request::extract_is_admin_from_old_log;

        #[test]
        fn typical_false() {
            let old_style_log = br#"{ gameId: 359005014,#012  clientId: 907,#012  account: #012   { accountId: 'fd7b99d56feb5a50',#012     displayName: 'John',#012     email: 'test@shmeppy.invalid' },#012  isAdmin: false,#012  log: #012   { debug: [Function: bound debug],#012     info: [Function: bound info],#012     alert: [Function: bound alert],#012     crit: [Function: bound crit],#012     error: [Function: bound error],#012     warning: [Function: bound warning],#012     notice: [Function: bound notice] },#012  socket: #012   WebSocket {#012     domain: null,#012     _events: { close: [Array], error: [Function] },#012     _eventsCount: 2,#012     _maxListeners: undefined,#012     readyState: 1,#012     protocol: '',#012     _binaryType: 'nodebuffer',#012     _closeFrameReceived: false,#012     _closeFrameSent: false,#012     _closeMessage: '',#012     _closeTimer: null,#012     _closeCode: 1006,#012     _extensions: {},#012     _isServer: true,#012     _receiver: #012      Receiver {#012        _writableState: [Object],#012        writable: true,#012        domain: null,#012        _events: [Object],#012        _eventsCount: 6,#012        _maxListeners: undefined,#012        _binaryType: 'nodebuffer',#012        _extensions: {},#012        _maxPayload: 104857600,#012        _bufferedBytes: 0,#012        _buffers: [],#012        _compressed: false,#012        _payloadLength: 40,#012        _mask: <Buffer 6f 08 94 34>,#012        _fragmented: 0,#012        _masked: true,#012        _fin: true,#012        _opcode: 1,#012        _totalPayloadLength: 0,#012        _messageLength: 0,#012        _fragments: [],#012        _state: 0,#012        _loop: false,#012        [Symbol(websocket)]: [Circular] },#012     _sender: #012      Sender {#012        _extensions: {},#012        _socket: [Object],#012        _firstFragment: true,#012        _compress: false,#012        _bufferedBytes: 0,#012        _deflating: false,#012        _queue: [] },#012     _socket: #012      TLSSocket {#012        _tlsOptions: [Object],#012        _secureEstablished: true,#012        _securePending: false,#012        _newSessionPending: false,#012        _controlReleased: true,#012        _SNICallback: null,#012        servername: 'shmeppy.com',#012        npnProtocol: false,#012        alpnProtocol: false,#012        authorized: false,#012        authorizationError: null,#012        encrypted: true,#012        _events: [Object],#012        _eventsCount: 9,#012        connecting: false,#012        _hadError: false,#012        _handle: [Object],#012        _parent: [Object],#012        _host: null,#012        _readableState: [Object],#012        readable: true,#012        domain: null,#012        _maxListeners: undefined,#012        _writableState: [Object],#012        writable: true,#012        allowHalfOpen: false,#012        _bytesDispatched: 180,#012        _sockname: null,#012        _pendingData: null,#012        _pendingEncoding: '',#012        server: [Object],#012        _server: null,#012        ssl: [Object],#012        _requestCert: false,#012        _rejectUnauthorized: true,#012        _idleTimeout: -1,#012        _idleNext: null,#012        _idlePrev: null,#012        _idleStart: 1145905219,#012        _destroyed: false,#012        parser: null,#012        on: [Function],#012        _paused: false,#012        [Symbol(asyncId)]: 8927446,#012        [Symbol(bytesRead)]: 0,#012        [Symbol(asyncId)]: 8927447,#012        [Symbol(triggerAsyncId)]: 8,#012        [Symbol(websocket)]: [Circular] },#012     log: #012      { debug: [Function: bound debug],#012        info: [Function: bound info],#012        alert: [Function: bound alert],#012        crit: [Function: bound crit],#012        error: [Function: bound error],#012        warning: [Function: bound warning],#012        notice: [Function: bound notice] } },#012  lastPing: 1558966491296 }"#;
            assert_eq!(
                extract_is_admin_from_old_log(old_style_log),
                Some(false));
        }

        #[test]
        fn typical_true() {
            let old_style_log = br#"{ gameId: 359005014,#012  clientId: 907,#012  account: #012   { accountId: 'fd7b99d56feb5a50',#012     displayName: 'John',#012     email: 'test@shmeppy.invalid' },#012  isAdmin: true,#012  log: #012   { debug: [Function: bound debug],#012     info: [Function: bound info],#012     alert: [Function: bound alert],#012     crit: [Function: bound crit],#012     error: [Function: bound error],#012     warning: [Function: bound warning],#012     notice: [Function: bound notice] },#012  socket: #012   WebSocket {#012     domain: null,#012     _events: { close: [Array], error: [Function] },#012     _eventsCount: 2,#012     _maxListeners: undefined,#012     readyState: 1,#012     protocol: '',#012     _binaryType: 'nodebuffer',#012     _closeFrameReceived: false,#012     _closeFrameSent: false,#012     _closeMessage: '',#012     _closeTimer: null,#012     _closeCode: 1006,#012     _extensions: {},#012     _isServer: true,#012     _receiver: #012      Receiver {#012        _writableState: [Object],#012        writable: true,#012        domain: null,#012        _events: [Object],#012        _eventsCount: 6,#012        _maxListeners: undefined,#012        _binaryType: 'nodebuffer',#012        _extensions: {},#012        _maxPayload: 104857600,#012        _bufferedBytes: 0,#012        _buffers: [],#012        _compressed: false,#012        _payloadLength: 40,#012        _mask: <Buffer 6f 08 94 34>,#012        _fragmented: 0,#012        _masked: true,#012        _fin: true,#012        _opcode: 1,#012        _totalPayloadLength: 0,#012        _messageLength: 0,#012        _fragments: [],#012        _state: 0,#012        _loop: false,#012        [Symbol(websocket)]: [Circular] },#012     _sender: #012      Sender {#012        _extensions: {},#012        _socket: [Object],#012        _firstFragment: true,#012        _compress: false,#012        _bufferedBytes: 0,#012        _deflating: false,#012        _queue: [] },#012     _socket: #012      TLSSocket {#012        _tlsOptions: [Object],#012        _secureEstablished: true,#012        _securePending: false,#012        _newSessionPending: false,#012        _controlReleased: true,#012        _SNICallback: null,#012        servername: 'shmeppy.com',#012        npnProtocol: false,#012        alpnProtocol: false,#012        authorized: false,#012        authorizationError: null,#012        encrypted: true,#012        _events: [Object],#012        _eventsCount: 9,#012        connecting: false,#012        _hadError: false,#012        _handle: [Object],#012        _parent: [Object],#012        _host: null,#012        _readableState: [Object],#012        readable: true,#012        domain: null,#012        _maxListeners: undefined,#012        _writableState: [Object],#012        writable: true,#012        allowHalfOpen: false,#012        _bytesDispatched: 180,#012        _sockname: null,#012        _pendingData: null,#012        _pendingEncoding: '',#012        server: [Object],#012        _server: null,#012        ssl: [Object],#012        _requestCert: false,#012        _rejectUnauthorized: true,#012        _idleTimeout: -1,#012        _idleNext: null,#012        _idlePrev: null,#012        _idleStart: 1145905219,#012        _destroyed: false,#012        parser: null,#012        on: [Function],#012        _paused: false,#012        [Symbol(asyncId)]: 8927446,#012        [Symbol(bytesRead)]: 0,#012        [Symbol(asyncId)]: 8927447,#012        [Symbol(triggerAsyncId)]: 8,#012        [Symbol(websocket)]: [Circular] },#012     log: #012      { debug: [Function: bound debug],#012        info: [Function: bound info],#012        alert: [Function: bound alert],#012        crit: [Function: bound crit],#012        error: [Function: bound error],#012        warning: [Function: bound warning],#012        notice: [Function: bound notice] } },#012  lastPing: 1558966491296 }"#;
            assert_eq!(
                extract_is_admin_from_old_log(old_style_log),
                Some(true));
        }

        #[test]
        fn missing() {
            // Same as above except I removed the isAdmin field
            let old_style_log = br#"{ gameId: 359005014,#012  clientId: 907,#012  account: #012   { accountId: 'fd7b99d56feb5a50',#012     displayName: 'John',#012     email: 'test@shmeppy.invalid' },#012  log: #012   { debug: [Function: bound debug],#012     info: [Function: bound info],#012     alert: [Function: bound alert],#012     crit: [Function: bound crit],#012     error: [Function: bound error],#012     warning: [Function: bound warning],#012     notice: [Function: bound notice] },#012  socket: #012   WebSocket {#012     domain: null,#012     _events: { close: [Array], error: [Function] },#012     _eventsCount: 2,#012     _maxListeners: undefined,#012     readyState: 1,#012     protocol: '',#012     _binaryType: 'nodebuffer',#012     _closeFrameReceived: false,#012     _closeFrameSent: false,#012     _closeMessage: '',#012     _closeTimer: null,#012     _closeCode: 1006,#012     _extensions: {},#012     _isServer: true,#012     _receiver: #012      Receiver {#012        _writableState: [Object],#012        writable: true,#012        domain: null,#012        _events: [Object],#012        _eventsCount: 6,#012        _maxListeners: undefined,#012        _binaryType: 'nodebuffer',#012        _extensions: {},#012        _maxPayload: 104857600,#012        _bufferedBytes: 0,#012        _buffers: [],#012        _compressed: false,#012        _payloadLength: 40,#012        _mask: <Buffer 6f 08 94 34>,#012        _fragmented: 0,#012        _masked: true,#012        _fin: true,#012        _opcode: 1,#012        _totalPayloadLength: 0,#012        _messageLength: 0,#012        _fragments: [],#012        _state: 0,#012        _loop: false,#012        [Symbol(websocket)]: [Circular] },#012     _sender: #012      Sender {#012        _extensions: {},#012        _socket: [Object],#012        _firstFragment: true,#012        _compress: false,#012        _bufferedBytes: 0,#012        _deflating: false,#012        _queue: [] },#012     _socket: #012      TLSSocket {#012        _tlsOptions: [Object],#012        _secureEstablished: true,#012        _securePending: false,#012        _newSessionPending: false,#012        _controlReleased: true,#012        _SNICallback: null,#012        servername: 'shmeppy.com',#012        npnProtocol: false,#012        alpnProtocol: false,#012        authorized: false,#012        authorizationError: null,#012        encrypted: true,#012        _events: [Object],#012        _eventsCount: 9,#012        connecting: false,#012        _hadError: false,#012        _handle: [Object],#012        _parent: [Object],#012        _host: null,#012        _readableState: [Object],#012        readable: true,#012        domain: null,#012        _maxListeners: undefined,#012        _writableState: [Object],#012        writable: true,#012        allowHalfOpen: false,#012        _bytesDispatched: 180,#012        _sockname: null,#012        _pendingData: null,#012        _pendingEncoding: '',#012        server: [Object],#012        _server: null,#012        ssl: [Object],#012        _requestCert: false,#012        _rejectUnauthorized: true,#012        _idleTimeout: -1,#012        _idleNext: null,#012        _idlePrev: null,#012        _idleStart: 1145905219,#012        _destroyed: false,#012        parser: null,#012        on: [Function],#012        _paused: false,#012        [Symbol(asyncId)]: 8927446,#012        [Symbol(bytesRead)]: 0,#012        [Symbol(asyncId)]: 8927447,#012        [Symbol(triggerAsyncId)]: 8,#012        [Symbol(websocket)]: [Circular] },#012     log: #012      { debug: [Function: bound debug],#012        info: [Function: bound info],#012        alert: [Function: bound alert],#012        crit: [Function: bound crit],#012        error: [Function: bound error],#012        warning: [Function: bound warning],#012        notice: [Function: bound notice] } },#012  lastPing: 1558966491296 }"#;
            assert_eq!(extract_is_admin_from_old_log(old_style_log), None);
        }

        #[test]
        fn empty() {
            assert_eq!(extract_is_admin_from_old_log(b""), None);
        }
    }

    mod extract_account_id_from_old_log {
        use crate::request::extract_account_id_from_old_log;

        #[test]
        fn typical() {
            let old_style_log = br#"{ gameId: 359005014,#012  clientId: 907,#012  account: #012   { accountId: 'fd7b99d56feb5a50',#012     displayName: 'John',#012     email: 'test@shmeppy.invalid' },#012  isAdmin: false,#012  log: #012   { debug: [Function: bound debug],#012     info: [Function: bound info],#012     alert: [Function: bound alert],#012     crit: [Function: bound crit],#012     error: [Function: bound error],#012     warning: [Function: bound warning],#012     notice: [Function: bound notice] },#012  socket: #012   WebSocket {#012     domain: null,#012     _events: { close: [Array], error: [Function] },#012     _eventsCount: 2,#012     _maxListeners: undefined,#012     readyState: 1,#012     protocol: '',#012     _binaryType: 'nodebuffer',#012     _closeFrameReceived: false,#012     _closeFrameSent: false,#012     _closeMessage: '',#012     _closeTimer: null,#012     _closeCode: 1006,#012     _extensions: {},#012     _isServer: true,#012     _receiver: #012      Receiver {#012        _writableState: [Object],#012        writable: true,#012        domain: null,#012        _events: [Object],#012        _eventsCount: 6,#012        _maxListeners: undefined,#012        _binaryType: 'nodebuffer',#012        _extensions: {},#012        _maxPayload: 104857600,#012        _bufferedBytes: 0,#012        _buffers: [],#012        _compressed: false,#012        _payloadLength: 40,#012        _mask: <Buffer 6f 08 94 34>,#012        _fragmented: 0,#012        _masked: true,#012        _fin: true,#012        _opcode: 1,#012        _totalPayloadLength: 0,#012        _messageLength: 0,#012        _fragments: [],#012        _state: 0,#012        _loop: false,#012        [Symbol(websocket)]: [Circular] },#012     _sender: #012      Sender {#012        _extensions: {},#012        _socket: [Object],#012        _firstFragment: true,#012        _compress: false,#012        _bufferedBytes: 0,#012        _deflating: false,#012        _queue: [] },#012     _socket: #012      TLSSocket {#012        _tlsOptions: [Object],#012        _secureEstablished: true,#012        _securePending: false,#012        _newSessionPending: false,#012        _controlReleased: true,#012        _SNICallback: null,#012        servername: 'shmeppy.com',#012        npnProtocol: false,#012        alpnProtocol: false,#012        authorized: false,#012        authorizationError: null,#012        encrypted: true,#012        _events: [Object],#012        _eventsCount: 9,#012        connecting: false,#012        _hadError: false,#012        _handle: [Object],#012        _parent: [Object],#012        _host: null,#012        _readableState: [Object],#012        readable: true,#012        domain: null,#012        _maxListeners: undefined,#012        _writableState: [Object],#012        writable: true,#012        allowHalfOpen: false,#012        _bytesDispatched: 180,#012        _sockname: null,#012        _pendingData: null,#012        _pendingEncoding: '',#012        server: [Object],#012        _server: null,#012        ssl: [Object],#012        _requestCert: false,#012        _rejectUnauthorized: true,#012        _idleTimeout: -1,#012        _idleNext: null,#012        _idlePrev: null,#012        _idleStart: 1145905219,#012        _destroyed: false,#012        parser: null,#012        on: [Function],#012        _paused: false,#012        [Symbol(asyncId)]: 8927446,#012        [Symbol(bytesRead)]: 0,#012        [Symbol(asyncId)]: 8927447,#012        [Symbol(triggerAsyncId)]: 8,#012        [Symbol(websocket)]: [Circular] },#012     log: #012      { debug: [Function: bound debug],#012        info: [Function: bound info],#012        alert: [Function: bound alert],#012        crit: [Function: bound crit],#012        error: [Function: bound error],#012        warning: [Function: bound warning],#012        notice: [Function: bound notice] } },#012  lastPing: 1558966491296 }"#;
            assert_eq!(
                extract_account_id_from_old_log(old_style_log),
                Some(0xfd7b99d56feb5a50));
        }

        #[test]
        fn missing() {
            // Same as above except I removed the account ID
            let old_style_log = br#"{ gameId: 359005014,#012  clientId: 907,#012  account: #012   { displayName: 'John',#012     email: 'test@shmeppy.invalid' },#012  isAdmin: false,#012  log: #012   { debug: [Function: bound debug],#012     info: [Function: bound info],#012     alert: [Function: bound alert],#012     crit: [Function: bound crit],#012     error: [Function: bound error],#012     warning: [Function: bound warning],#012     notice: [Function: bound notice] },#012  socket: #012   WebSocket {#012     domain: null,#012     _events: { close: [Array], error: [Function] },#012     _eventsCount: 2,#012     _maxListeners: undefined,#012     readyState: 1,#012     protocol: '',#012     _binaryType: 'nodebuffer',#012     _closeFrameReceived: false,#012     _closeFrameSent: false,#012     _closeMessage: '',#012     _closeTimer: null,#012     _closeCode: 1006,#012     _extensions: {},#012     _isServer: true,#012     _receiver: #012      Receiver {#012        _writableState: [Object],#012        writable: true,#012        domain: null,#012        _events: [Object],#012        _eventsCount: 6,#012        _maxListeners: undefined,#012        _binaryType: 'nodebuffer',#012        _extensions: {},#012        _maxPayload: 104857600,#012        _bufferedBytes: 0,#012        _buffers: [],#012        _compressed: false,#012        _payloadLength: 40,#012        _mask: <Buffer 6f 08 94 34>,#012        _fragmented: 0,#012        _masked: true,#012        _fin: true,#012        _opcode: 1,#012        _totalPayloadLength: 0,#012        _messageLength: 0,#012        _fragments: [],#012        _state: 0,#012        _loop: false,#012        [Symbol(websocket)]: [Circular] },#012     _sender: #012      Sender {#012        _extensions: {},#012        _socket: [Object],#012        _firstFragment: true,#012        _compress: false,#012        _bufferedBytes: 0,#012        _deflating: false,#012        _queue: [] },#012     _socket: #012      TLSSocket {#012        _tlsOptions: [Object],#012        _secureEstablished: true,#012        _securePending: false,#012        _newSessionPending: false,#012        _controlReleased: true,#012        _SNICallback: null,#012        servername: 'shmeppy.com',#012        npnProtocol: false,#012        alpnProtocol: false,#012        authorized: false,#012        authorizationError: null,#012        encrypted: true,#012        _events: [Object],#012        _eventsCount: 9,#012        connecting: false,#012        _hadError: false,#012        _handle: [Object],#012        _parent: [Object],#012        _host: null,#012        _readableState: [Object],#012        readable: true,#012        domain: null,#012        _maxListeners: undefined,#012        _writableState: [Object],#012        writable: true,#012        allowHalfOpen: false,#012        _bytesDispatched: 180,#012        _sockname: null,#012        _pendingData: null,#012        _pendingEncoding: '',#012        server: [Object],#012        _server: null,#012        ssl: [Object],#012        _requestCert: false,#012        _rejectUnauthorized: true,#012        _idleTimeout: -1,#012        _idleNext: null,#012        _idlePrev: null,#012        _idleStart: 1145905219,#012        _destroyed: false,#012        parser: null,#012        on: [Function],#012        _paused: false,#012        [Symbol(asyncId)]: 8927446,#012        [Symbol(bytesRead)]: 0,#012        [Symbol(asyncId)]: 8927447,#012        [Symbol(triggerAsyncId)]: 8,#012        [Symbol(websocket)]: [Circular] },#012     log: #012      { debug: [Function: bound debug],#012        info: [Function: bound info],#012        alert: [Function: bound alert],#012        crit: [Function: bound crit],#012        error: [Function: bound error],#012        warning: [Function: bound warning],#012        notice: [Function: bound notice] } },#012  lastPing: 1558966491296 }"#;
            assert_eq!(extract_account_id_from_old_log(old_style_log), None);
        }

        #[test]
        fn empty() {
            assert_eq!(extract_account_id_from_old_log(b""), None);
        }
    }

    mod parse_message {
        use crate::request::{parse_message, Message::*};
        use crate::parse::UUID;

        #[test]
        fn no_match() {
            assert_eq!(parse_message(b"Committed 1 operation(s)."), None);
        }

        #[test]
        fn started_game_id() {
            assert_eq!(
                parse_message(b"Started websocket: GET /game-socket/92829"),
                Some(StartedGameId(92829)));
            assert_eq!(
                parse_message(b"Started websocket: GET /game-socket/92829?version=1"),
                Some(StartedGameId(92829)));
            assert_eq!(
                parse_message(b"Started websocket: GET /game-socket/92829?version=1&lastSeenIndex=101"),
                Some(StartedGameId(92829)));
        }

        #[test]
        fn analytics_id() {
            assert_eq!(
                parse_message(b"Analytics ID: 00000000-0000-0000-0000-000000000001"),
                Some(AnalyticsId(UUID(1))));
            assert_eq!(
                parse_message(b"Analytics ID: b47c855d-48dc-4ac8-814b-b6200d2eac43"),
                Some(AnalyticsId(UUID(0xb47c855d_48dc_4ac8_814b_b6200d2eac43))));
        }

        #[test]
        fn authenticated_as_old_log() {
            assert_eq!(
                parse_message(br#"Client added to client DB: { gameId: 1109643912,#012  clientId: 672,#012  account: #012   { accountId: 'ab0802483d806ed3',#012     displayName: 'John',#012     email: 'john@shmeppy.invalid' },#012  isAdmin: true,#012  log: #012   { debug: [Function: bound debug],#012     info: [Function: bound info],#012     alert: [Function: bound alert],#012     crit: [Function: bound crit],#012     error: [Function: bound error],#012     warning: [Function: bound warning],#012     notice: [Function: bound notice] },#012  socket: #012   WebSocket {#012     domain: null,#012     _events: { close: [Array], error: [Function] },#012     _eventsCount: 2,#012     _maxListeners: undefined,#012     readyState: 1,#012     protocol: '',#012     _binaryType: 'nodebuffer',#012     _closeFrameReceived: false,#012     _closeFrameSent: false,#012     _closeMessage: '',#012     _closeTimer: null,#012     _closeCode: 1006,#012     _extensions: {},#012     _isServer: true,#012     _receiver: #012      Receiver {#012        _writableState: [Object],#012        writable: true,#012        domain: null,#012        _events: [Object],#012        _eventsCount: 6,#012        _maxListeners: undefined,#012        _binaryType: 'nodebuffer',#012        _extensions: {},#012        _maxPayload: 104857600,#012        _bufferedBytes: 0,#012        _buffers: [],#012        _compressed: false,#012        _payloadLength: 40,#012        _mask: <Buffer 43 5c 66 14>,#012        _fragmented: 0,#012        _masked: true,#012        _fin: true,#012        _opcode: 1,#012        _totalPayloadLength: 0,#012        _messageLength: 0,#012        _fragments: [],#012        _state: 0,#012        _loop: false,#012        [Symbol(websocket)]: [Circular] },#012     _sender: #012      Sender {#012        _extensions: {},#012        _socket: [Object],#012        _firstFragment: true,#012        _compress: false,#012        _bufferedBytes: 0,#012        _deflating: false,#012        _queue: [] },#012     _socket: #012      TLSSocket {#012        _tlsOptions: [Object],#012        _secureEstablished: true,#012        _securePending: false,#012        _newSessionPending: false,#012        _controlReleased: true,#012        _SNICallback: null,#012        servername: 'shmeppy.com',#012        npnProtocol: false,#012        alpnProtocol: false,#012        authorized: false,#012        authorizationError: null,#012        encrypted: true,#012        _events: [Object],#012        _eventsCount: 9,#012        connecting: false,#012        _hadError: false,#012        _handle: [Object],#012        _parent: [Object],#012        _host: null,#012        _readableState: [Object],#012        readable: true,#012        domain: null,#012        _maxListeners: undefined,#012        _writableState: [Object],#012        writable: true,#012        allowHalfOpen: false,#012        _bytesDispatched: 180,#012        _sockname: null,#012        _pendingData: null,#012        _pendingEncoding: '',#012        server: [Object],#012        _server: null,#012        ssl: [Object],#012        _requestCert: false,#012        _rejectUnauthorized: true,#012        _idleTimeout: -1,#012        _idleNext: null,#012        _idlePrev: null,#012        _idleStart: 279999795,#012        _destroyed: false,#012        parser: null,#012        on: [Function],#012        _paused: false,#012        [Symbol(asyncId)]: 1953772,#012        [Symbol(bytesRead)]: 0,#012        [Symbol(asyncId)]: 1953774,#012        [Symbol(triggerAsyncId)]: 8,#012        [Symbol(websocket)]: [Circular] },#012     log: #012      { debug: [Function: bound debug],#012        info: [Function: bound info],#012        alert: [Function: bound alert],#012        crit: [Function: bound crit],#012        error: [Function: bound error],#012        warning: [Function: bound warning],#012        notice: [Function: bound notice] } },#012  lastPing: 1555912228425 }"#),
                Some(AuthenticatedAs(0xab0802483d806ed3, true)));
        }

        #[test]
        fn authenticated_as() {
            assert_eq!(
                parse_message(br#"Client added to client DB: {"gameId":302078290,"clientId":12681,"account":{"accountId":"df547ed38259c164","displayName":"John"},"isAdmin":false,"initialLastSeenIndex":679,"lastPing":1590637288548}"#),
                Some(AuthenticatedAs(0xdf547ed38259c164, false)));

            // I messed with the contents of these to give the deserialization
            // some extra testing since those aren't in their own functions
            // with their own unit tests.
            assert_eq!(
                // accountId is the wrong type
                parse_message(br#"Client added to client DB: {"gameId":302078290,"clientId":12681,"account":{"accountId":77,"displayName":"John"},"isAdmin":false,"initialLastSeenIndex":679,"lastPing":1590637288548}"#),
                None);
            assert_eq!(
                // isAdmin is missing
                parse_message(br#"Client added to client DB: {"gameId":302078290,"clientId":12681,"account":{"accountId":"df547ed38259c164","displayName":"John"},"initialLastSeenIndex":679,"lastPing":1590637288548}"#),
                None);
            assert_eq!(
                // The whole object is empty
                parse_message(br#"Client added to client DB: {}"#),
                None);
            assert_eq!(
                // There's nothing there
                parse_message(br#"Client added to client DB: "#),
                None);
        }

        #[test]
        fn finished() {
            assert_eq!(parse_message(b"Finished websocket: 1005 ''"),
                       Some(Finished));
            assert_eq!(parse_message(b"Finished: 200 'OK'"), Some(Finished));
            assert_eq!(parse_message(b"Waiting for client to complete closing handshake."), Some(Finished));
        }
    }

    mod extract_integer {
        use crate::request::extract_integer;

        #[test]
        fn simple_u8() {
            assert_eq!(extract_integer::<u8>(b"12"), Some(12));
            assert_eq!(extract_integer::<u8>(b"12 hello"), Some(12));
            assert_eq!(extract_integer::<u8>(b"0 hello"), Some(0));
            assert_eq!(extract_integer::<u8>(b"255 hello"), Some(255));
        }

        #[test]
        fn overflow_u8() {
            assert_eq!(extract_integer::<u8>(b"256 hello"), None);
        }

        #[test]
        fn space_in_front() {
            assert_eq!(extract_integer::<u8>(b" 12 hello"), None);
        }

        #[test]
        fn empty() {
            assert_eq!(extract_integer::<u8>(b""), None);
        }
    }

    mod skip_prefix {
        use crate::request::skip_prefix;

        #[test]
        fn prefix_present() {
            let text = b"HelloWorld";
            let prefix = b"Hello";
            let result = skip_prefix(prefix, text);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), b"World");
            assert_eq!(result.unwrap().as_ptr(), text[5..].as_ptr());
        }

        #[test]
        fn prefix_missing() {
            let text = b"HelloWorld";
            let prefix = b"Goodbye";
            let result = skip_prefix(prefix, text);
            assert!(result.is_none());
        }

        #[test]
        fn empty_prefix() {
            let text = b"HelloWorld";
            let prefix = b"";
            let result = skip_prefix(prefix, text);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), b"HelloWorld");
            assert_eq!(result.unwrap().as_ptr(), text.as_ptr());
        }

        #[test]
        fn empty_text() {
            let text = b"";
            let prefix = b"Hello";
            let result = skip_prefix(prefix, text);
            assert!(result.is_none());
        }

        #[test]
        fn empty_both() {
            let text = b"";
            let prefix = b"";
            let result = skip_prefix(prefix, text);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), b"");
            assert_eq!(result.unwrap().as_ptr(), text.as_ptr());
        }
    }
}
