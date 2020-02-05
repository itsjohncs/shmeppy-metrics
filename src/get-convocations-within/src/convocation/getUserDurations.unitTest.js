import assert from "assert";

import getUserDurations from "./getUserDurations.js";
import TimeSpan from "./TimeSpan.js";


function _minutes(v) {
    return v * 1000 * 60;
}


class FakeUserPresence {
    constructor({during, accountId}) {
        this.during = during;
        this._accountId = accountId;
    }

    getAccountId() {
        return this._accountId;
    }
}


describe("getUserDurations", function() {
    it("returns a single user's duration in simplest case", function() {
        const timeSpan = new TimeSpan(
            new Date("1/1/2019 1:00:00 PM"),
            new Date("1/1/2019 1:10:00 PM"));
        const presences = [
            new FakeUserPresence({
                accountId: "a",
                during: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")),
            }),
        ];
        assert.deepStrictEqual(
            getUserDurations({timeSpan, presences}),
            new Map([["a", _minutes(10)]]));
    });

    it("correctly adds multiple user durations", function() {
        const timeSpan = new TimeSpan(
            new Date("1/1/2019 1:00:00 PM"),
            new Date("1/1/2019 1:10:00 PM"));
        const presences = [
            new FakeUserPresence({
                accountId: "a",
                during: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:02:00 PM")),
            }),
            new FakeUserPresence({
                accountId: "a",
                during: new TimeSpan(
                    new Date("1/1/2019 1:05:00 PM"),
                    new Date("1/1/2019 1:07:00 PM")),
            }),
        ];
        assert.deepStrictEqual(
            getUserDurations({timeSpan, presences}),
            new Map([["a", _minutes(4)]]));
    });

    it("correctly limits user duration to timeSpan", function() {
        const timeSpan = new TimeSpan(
            new Date("1/1/2019 1:00:00 PM"),
            new Date("1/1/2019 1:10:00 PM"));
        const presences = [
            new FakeUserPresence({
                accountId: "a",
                during: new TimeSpan(
                    new Date("1/1/2019 12:00:00 PM"),
                    new Date("1/1/2019 2:00:00 PM")),
            }),
        ];
        assert.deepStrictEqual(
            getUserDurations({timeSpan, presences}),
            new Map([["a", _minutes(10)]]));
    });

    it("correctly considers separate users", function() {
        const timeSpan = new TimeSpan(
            new Date("1/1/2019 1:00:00 PM"),
            new Date("1/1/2019 1:10:00 PM"));
        const presences = [
            new FakeUserPresence({
                accountId: "a",
                during: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")),
            }),
            new FakeUserPresence({
                accountId: "b",
                during: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:05:00 PM")),
            }),
        ];
        assert.deepStrictEqual(
            getUserDurations({timeSpan, presences}),
            new Map([["a", _minutes(10)], ["b", _minutes(5)]]));
    });
});
