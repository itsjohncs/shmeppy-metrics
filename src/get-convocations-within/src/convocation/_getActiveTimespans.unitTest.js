import assert from "assert";

import _getActiveTimespans, {_ACTIVITY_WINDOW_MINUTES, _addMinutes} from "./_getActiveTimespans.js";
import TimeSpan from "./TimeSpan.js";
import {GameRequest} from "../parse/GameRequest.js";
import {LogRecord} from "../parse/LogRecord.js";


function _makeTestGameRequest(records) {
    return new GameRequest({
        gameId: 1,
        accountId: "someAccountId",
        requestId: "someRequestId",
        records,
    });
}


function _makeActivityRecord(requestId, datetime) {
    return new LogRecord({
        requestId,
        datetime,
        message: "Committed 1 operation(s).",
    });
}


describe("_addMinutes", function() {
    it("is identity if 0 is passed in as second argument", function() {
        assert.deepStrictEqual(
            _addMinutes(new Date("10/20/1992 10:00:00"), 0),
            new Date("10/20/1992 10:00:00"));
    });

    it("can add minutes", function() {
        const datetime = new Date("10/20/1992 10:00:00");
        assert.deepStrictEqual(
            _addMinutes(new Date("10/20/1992 10:00:00"), 10),
            new Date("10/20/1992 10:10:00"));
    });

    it("can subtract minutes", function() {
        const datetime = new Date("10/20/1992 10:00:00");
        assert.deepStrictEqual(
            _addMinutes(new Date("10/20/1992 10:00:00"), -10),
            new Date("10/20/1992 9:50:00"));
    });
});


describe("_getActiveTimespans", function() {
    it("returns an empty list if given one", function() {
        assert.deepStrictEqual(_getActiveTimespans([]), []);
    });

    it("returns a single correctly-sized timespan for a single operation", function() {
        const testRequest = _makeTestGameRequest([
            _makeActivityRecord(1, new Date("10/20/1992 10:00:00")),
        ]);

        assert.strictEqual(_ACTIVITY_WINDOW_MINUTES, 60);
        assert.deepStrictEqual(
            _getActiveTimespans([testRequest]),
            [
                new TimeSpan(
                    new Date("10/20/1992 9:30:00"),
                    new Date("10/20/1992 10:30:00")),
            ]);
    });

    it("merges two timespans that are close to eachother in the same request", function() {
        const testRequest = _makeTestGameRequest([
            _makeActivityRecord(1, new Date("10/20/1992 10:00:00")),
            _makeActivityRecord(1, new Date("10/20/1992 10:10:00")),
        ]);
        assert.strictEqual(_ACTIVITY_WINDOW_MINUTES, 60);
        assert.deepStrictEqual(
            _getActiveTimespans([testRequest]),
            [
                new TimeSpan(
                    new Date("10/20/1992 9:30:00"),
                    new Date("10/20/1992 10:40:00"))
            ])
    });

    it("merges three timespans that are close to eachother in the same request", function() {
        const testRequest = _makeTestGameRequest([
            _makeActivityRecord(1, new Date("10/20/1992 10:00:00")),
            _makeActivityRecord(1, new Date("10/20/1992 10:10:00")),
            _makeActivityRecord(1, new Date("10/20/1992 9:55:00")),
        ]);
        assert.strictEqual(_ACTIVITY_WINDOW_MINUTES, 60);
        assert.deepStrictEqual(
            _getActiveTimespans([testRequest]),
            [
                new TimeSpan(
                    new Date("10/20/1992 9:25:00"),
                    new Date("10/20/1992 10:40:00"))
            ]);
    });

    it("merges two timespans that are close to eachother in separate requests", function() {
        const testRequests = [
            _makeTestGameRequest([
                _makeActivityRecord(1, new Date("10/20/1992 10:00:00")),
            ]),
            _makeTestGameRequest([
                _makeActivityRecord(1, new Date("10/20/1992 10:10:00")),
            ]),
        ];
        assert.strictEqual(_ACTIVITY_WINDOW_MINUTES, 60);
        assert.deepStrictEqual(
            _getActiveTimespans(testRequests),
            [
                new TimeSpan(
                    new Date("10/20/1992 9:30:00"),
                    new Date("10/20/1992 10:40:00"))
            ]);
    });
});
