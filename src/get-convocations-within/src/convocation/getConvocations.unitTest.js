import assert from "assert";

import getConvocations from "./getConvocations.js";
import {GameRequest} from "../parse/GameRequest.js";
import {LogRecord} from "../parse/LogRecord.js";
import TimeSpan from "./TimeSpan.js";


describe("getConvocations", function() {
    it("ignores forever logged-in users without activity", function() {
        // Two users in a game pretty much all day, but no operations are
        // logged.
        const gameRequests = [
            new GameRequest({
                gameId: 1,
                accountId: "A",
                requestId: "a",
                websocketOpenedDatetime: new Date("10/20/1992 1:00:00 AM"),
                websocketClosedDatetime: new Date("10/20/1992 11:00:00 PM"),
                records: [
                ],
            }),
            new GameRequest({
                gameId: 1,
                accountId: "B",
                requestId: "b",
                websocketOpenedDatetime: new Date("10/20/1992 3:00:00 AM"),
                websocketClosedDatetime: new Date("10/20/1992 9:00:00 PM"),
                records: [
                ],
            }),
        ];

        const convocations = getConvocations({gameRequests});
        assert.deepStrictEqual(convocations, []);
    });

    it("finds simple two-person convocation", function() {
        // Two users in a game pretty much all day. One operation is committed
        // in the middle of the day. We should get one convocation for an
        // hour centered around that operation commit time.
        const gameRequests = [
            new GameRequest({
                gameId: 1,
                accountId: "A",
                requestId: "a",
                websocketOpenedDatetime: new Date("10/20/1992 1:00:00 AM"),
                websocketClosedDatetime: new Date("10/20/1992 11:00:00 PM"),
                records: [
                    new LogRecord({
                        requestId: "a",
                        datetime: new Date("10/20/1992 12:00:00 PM"),
                        message: "Committed 1 operation(s).",
                    }),
                ],
            }),
            new GameRequest({
                gameId: 1,
                accountId: "B",
                requestId: "b",
                websocketOpenedDatetime: new Date("10/20/1992 3:00:00 AM"),
                websocketClosedDatetime: new Date("10/20/1992 9:00:00 PM"),
                records: [
                ],
            }),
        ];

        const convocations = getConvocations({gameRequests});
        assert.strictEqual(convocations.length, 1);
        assert.deepStrictEqual(
            convocations[0].timeSpan,
            new TimeSpan(
                new Date("10/20/1992 11:30:00 AM"),
                new Date("10/20/1992 12:30:00 PM")),
        );
    });
});
