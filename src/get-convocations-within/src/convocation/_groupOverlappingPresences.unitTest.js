import assert from "assert";

import _groupOverlappingPresences from "./_groupOverlappingPresences.js";
import TimeSpan from "./TimeSpan.js";
import UserPresence from "./UserPresence.js";


const FAKE_REQUEST = Symbol("FAKE_REQUEST");


function _makeTestPresence(during, requests = [FAKE_REQUEST]) {
    return new UserPresence({during, requests});
}


describe("_groupOverlappingPresences", function() {
    it("returns an empty list if given one", function() {
        assert.deepStrictEqual(_groupOverlappingPresences([]), []);
    });

    it("returns nothing if no presences overlap", function() {
        const presences = [
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:10:00 PM"),
                             new Date("1/1/2019 1:20:00 PM"))),
        ];

        assert.deepStrictEqual(_groupOverlappingPresences(presences), []);
    });

    it("returns pair of overlapping requests together", function() {
        const presences = [
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:05:00 PM"),
                             new Date("1/1/2019 1:15:00 PM"))),
        ];

        assert.deepStrictEqual(
            _groupOverlappingPresences(presences),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:05:00 PM"),
                        new Date("1/1/2019 1:10:00 PM")),
                    presences,
                },
            ]);
    });

    it("detects separate groups with common request", function() {
        const presences = [
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:05:00 PM"),
                             new Date("1/1/2019 1:15:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:10:00 PM"),
                             new Date("1/1/2019 1:15:00 PM"))),
        ];

        assert.deepStrictEqual(
            _groupOverlappingPresences(presences),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:05:00 PM"),
                        new Date("1/1/2019 1:10:00 PM")),
                    presences: [presences[0], presences[1]]
                },
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:10:00 PM"),
                        new Date("1/1/2019 1:15:00 PM")),
                    presences: [presences[1], presences[2]]
                },
            ]);
    });

    it("detects separate groups without request", function() {
        const presences = [
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:05:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:10:00 PM"),
                             new Date("1/1/2019 1:20:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:15:00 PM"),
                             new Date("1/1/2019 1:20:00 PM"))),
        ];

        assert.deepStrictEqual(
            _groupOverlappingPresences(presences),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:05:00 PM"),
                        new Date("1/1/2019 1:10:00 PM")),
                    presences: [presences[0], presences[1]]
                },
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:15:00 PM"),
                        new Date("1/1/2019 1:20:00 PM")),
                    presences: [presences[2], presences[3]]
                },
            ]);
    });

    it("detects group contained within another", function() {
        const presences = [
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:00:00 PM"),
                             new Date("1/1/2019 1:10:00 PM"))),
            _makeTestPresence(
                new TimeSpan(new Date("1/1/2019 1:03:00 PM"),
                             new Date("1/1/2019 1:06:00 PM"))),
        ];

        assert.deepStrictEqual(
            _groupOverlappingPresences(presences),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:00:00 PM"),
                        new Date("1/1/2019 1:03:00 PM")),
                    presences: [presences[0], presences[1]]
                },
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:03:00 PM"),
                        new Date("1/1/2019 1:06:00 PM")),
                    presences: [presences[0], presences[1], presences[2]]
                },
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:06:00 PM"),
                        new Date("1/1/2019 1:10:00 PM")),
                    presences: [presences[0], presences[1]]
                },
            ]);
    });
});
