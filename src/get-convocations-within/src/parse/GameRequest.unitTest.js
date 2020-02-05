import assert from "assert";

import {makeGameRequest} from "./GameRequest.js";
import {LogRecord} from "./LogRecord.js";


describe("makeGameRequest", function() {
    const RECORDS = [
        new LogRecord({
            requestId: "a",
            datetime: new Date("5/13/2019 7:42:00 PM"),
            message: "Started websocket: GET /game-socket/1 1.1",
        }),
        new LogRecord({
            requestId: "a",
            datetime: new Date("5/13/2019 7:42:00 PM"),
            message: (
                "Successfully authenticated as: { accountId: 'z',#012" +
                "  displayName: 'User',#012" +
                "  email: 'username@gmail.invalid' }")
        }),
        new LogRecord({
            requestId: "a",
            datetime: new Date("5/13/2019 7:50:00 PM"),
            message: "Finished websocket: 1006 ''",
        }),
    ];

    it("returns null if any data is unavailable", function() {
        assert.strictEqual(makeGameRequest(RECORDS.slice(0, 2)), null);
    });

    it("datapoints are extracted correctly", function() {
        const gameRequest = makeGameRequest(RECORDS);
        assert.strictEqual(gameRequest.gameId, 1);
        assert.strictEqual(gameRequest.accountId, "z");
        assert.strictEqual(gameRequest.requestId, "a");
        assert.strictEqual(gameRequest.websocketOpenedDatetime.getTime(),
                           new Date("5/13/2019 7:42:00 PM").getTime());
        assert.strictEqual(gameRequest.websocketClosedDatetime.getTime(),
                           new Date("5/13/2019 7:50:00 PM").getTime());
    });
});
