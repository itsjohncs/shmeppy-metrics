import assert from "assert";

import UserPresence from "./UserPresence.js";
import TimeSpan from "./TimeSpan.js";
import {GameRequest} from "../parse/GameRequest.js";


function _makeTestGameRequest(obj) {
    return new GameRequest({
        gameId: 1,
        accountId: "someAccountId",
        requestId: "someRequestId",
        records: [],
        ...obj,
    });
}


describe("UserPresence", function() {
    it("can be created from a single request", function() {
        const request = _makeTestGameRequest({
            websocketOpenedDatetime: new Date("1/1/2019 1:00:00 PM"),
            websocketClosedDatetime: new Date("1/1/2019 1:10:00 PM"),
        });
        assert.deepStrictEqual(
            UserPresence.fromSingle(request),
            new UserPresence({
                during: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")),
                requests: [request]
            }));
    });

    it("can be combined with other presences", function() {
        const requestA = _makeTestGameRequest({
            websocketOpenedDatetime: new Date("1/1/2019 1:00:00 PM"),
            websocketClosedDatetime: new Date("1/1/2019 1:10:00 PM"),
        });
        const requestB = _makeTestGameRequest({
            websocketOpenedDatetime: new Date("1/1/2019 1:05:00 PM"),
            websocketClosedDatetime: new Date("1/1/2019 1:15:00 PM"),
        });
        assert.deepStrictEqual(
            UserPresence.combined(
                UserPresence.fromSingle(requestA),
                UserPresence.fromSingle(requestB)),
            new UserPresence({
                during: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:15:00 PM")),
                requests: [requestA, requestB],
            }));
    });

    describe("fromMany", function() {
        it("returns an empty list if given one", function() {
            assert.deepStrictEqual(UserPresence.fromMany([]), []);
        });

        it("returns separate presences if requests don't overlap", function() {
            const requests = [
                _makeTestGameRequest({
                    websocketOpenedDatetime: new Date("1/1/2019 1:00:00 PM"),
                    websocketClosedDatetime: new Date("1/1/2019 1:10:00 PM"),
                }),
                _makeTestGameRequest({
                    websocketOpenedDatetime: new Date("1/1/2019 1:10:00 PM"),
                    websocketClosedDatetime: new Date("1/1/2019 1:20:00 PM"),
                }),
            ];
            assert.deepStrictEqual(
                UserPresence.fromMany(requests),
                [
                    UserPresence.fromSingle(requests[0]),
                    UserPresence.fromSingle(requests[1]),
                ]);
        });

        it("returns single presence if requests overlap", function() {
            const requests = [
                _makeTestGameRequest({
                    websocketOpenedDatetime: new Date("1/1/2019 1:00:00 PM"),
                    websocketClosedDatetime: new Date("1/1/2019 1:10:00 PM"),
                }),
                _makeTestGameRequest({
                    websocketOpenedDatetime: new Date("1/1/2019 1:05:00 PM"),
                    websocketClosedDatetime: new Date("1/1/2019 1:15:00 PM"),
                }),
            ];
            assert.deepStrictEqual(
                UserPresence.fromMany(requests),
                [
                    new UserPresence({
                        during: new TimeSpan(
                            new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:15:00 PM")),
                        requests,
                    }),
                ]);
        });
    });
});
