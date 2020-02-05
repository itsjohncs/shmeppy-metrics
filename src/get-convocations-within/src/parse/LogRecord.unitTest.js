import assert from "assert";

import {makeLogRecord} from "./LogRecord.js";


describe("makeLogRecord", function() {
    it("returns null on unrecognized format", function() {
        assert.strictEqual(makeLogRecord("invalid"), null);
    });

    describe("log format v1", function() {
        it("returns null", function() {
            assert.strictEqual(
                makeLogRecord("Oct  1 20:43:45 shmeppy-0 shmeppy-app: (c4dfd175-c0ff-43d5-bd06-6366fd701030) [INFO - 10/1/2018 8:43:45 PM] Finished websocket: 1001 ''"),
                null);
        });
    });

    describe("log format v2", function() {
        const v2LogLine = "shmeppy-0 shmeppy-app: (445204d1-2cf6-4a13-81a2-895e2ddfde86) [INFO - 10/1/2018 8:50:39 PM] Finished websocket: 1001 ''"

        it("parses date", function() {
            const expectedDatetime = new Date(Date.UTC(2018, 9, 1, 20, 50, 39, 0));
            const {datetime: actualDatetime} = makeLogRecord(v2LogLine);
            assert.strictEqual(actualDatetime.getTime(),
                               expectedDatetime.getTime());
        });

        it("parses request ID", function() {
            const {requestId: actualRequestId} = makeLogRecord(v2LogLine);
            assert.strictEqual(actualRequestId,
                               "445204d1-2cf6-4a13-81a2-895e2ddfde86");
        });

        it("parses message", function() {
            const {message: actualMessage} = makeLogRecord(v2LogLine);
            assert.strictEqual(actualMessage, "Finished websocket: 1001 ''");
        });
    });
});
