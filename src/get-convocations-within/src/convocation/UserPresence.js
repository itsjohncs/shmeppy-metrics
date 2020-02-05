import assert from "assert";

import groupBy from "../util/groupBy.js";
import TimeSpan from "./TimeSpan.js";



/**
 * Represents a user's presence in a game during a timespan.
 *
 * This abstracts away (potentially) multiple requests into a single timespan.
 */
class UserPresence {
    constructor({during, requests}) {
        assert(requests.length > 0);
        assert(requests.every(i => i.accountId === requests[0].accountId));
        assert(requests.every(i => i.gameId === requests[0].gameId));
        this.during = during;
        this.requests = requests;
    }

    getAccountId() {
        return this.requests[0].accountId;
    }

    getGameId() {
        return this.requests[0].gameId;
    }

    static fromSingle(request) {
        return new UserPresence({
            during: new TimeSpan(
                request.websocketOpenedDatetime,
                request.websocketClosedDatetime),
            requests: [request],
        });
    }

    static fromMany(gameRequests) {
        if (gameRequests.length === 0) {
            return [];
        }

        const groupedRequests = groupBy(gameRequests, "accountId").values();
        return (
            Array.from(groupedRequests)
                 .map(_requestsToUserPresences)
                 .reduce((a, b) => [...a, ...b]));
    }

    static combined(a, b) {
        return new UserPresence({
            during: TimeSpan.fill(a.during, b.during),
            requests: [...a.requests, ...b.requests],
        });
    }
}


function _sortedGameRequests(unsortedGameRequests) {
    return [...unsortedGameRequests].sort(
        function ({websocketOpenedDatetime: a}, {websocketOpenedDatetime: b}) {
            if (a > b) {
                return 1;
            } else if (a < b) {
                return -1;
            }

            return 0;
        });
}


/**
 * Transforms requests into UserPresences, handling overlapping requests.
 */
function _requestsToUserPresences(unsortedGameRequests) {
    assert(unsortedGameRequests.length > 0);
    assert(unsortedGameRequests.every(
        r => r.accountId === unsortedGameRequests[0].accountId));
    assert(unsortedGameRequests.every(
        r => r.gameId === unsortedGameRequests[0].gameId));

    // First get them in order so any overlapping game requests are next to
    // eachother.
    const gameRequests = _sortedGameRequests(unsortedGameRequests);

    const presences = [UserPresence.fromSingle(gameRequests[0])]
    for (let i = 1; i < gameRequests.length; ++i) {
        const lastPresence = presences[presences.length - 1];
        const requestPresence = UserPresence.fromSingle(gameRequests[i]);
        if (lastPresence.during.isTouching(requestPresence.during)) {
            presences[presences.length - 1] = UserPresence.combined(
                lastPresence, requestPresence);
        } else {
            presences.push(requestPresence);
        }
    }

    return presences;
}


export default UserPresence;
