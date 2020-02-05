import assert from "assert";

import _joinContiguousGroups from "./_joinContiguousGroups.js";
import TimeSpan from "./TimeSpan.js";


describe("_joinContiguousGroups", function() {
    it("returns an empty list if given one", function() {
        assert.deepStrictEqual(_joinContiguousGroups([]), []);
    });

    it("joins two groups less than an hour apart", function() {
        const groups = [
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:03:00 PM")),
                presences: [0]
            },
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 2:00:00 PM"),
                    new Date("1/1/2019 2:30:00 PM")),
                presences: [1]
            },
        ];

        assert.deepStrictEqual(
            _joinContiguousGroups(groups),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:00:00 PM"),
                        new Date("1/1/2019 2:30:00 PM")),
                    presences: [0, 1],
                }
            ]);
    });

    it("doesn't join two groups more than an hour apart", function() {
        const groups = [
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:03:00 PM")),
                presences: [0]
            },
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 3:00:00 PM"),
                    new Date("1/1/2019 3:30:00 PM")),
                presences: [1]
            },
        ];

        assert.deepStrictEqual(
            _joinContiguousGroups(groups),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:00:00 PM"),
                        new Date("1/1/2019 1:03:00 PM")),
                    presences: [0]
                },
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 3:00:00 PM"),
                        new Date("1/1/2019 3:30:00 PM")),
                    presences: [1]
                },
            ]);
    });

    it("order doesn't matter", function() {
        const groups = [
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:03:00 PM")),
                presences: [0]
            },
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 2:00:00 PM"),
                    new Date("1/1/2019 2:30:00 PM")),
                presences: [1]
            },
        ];

        const expectedResult = [
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 2:30:00 PM")),
                presences: [0, 1],
            }
        ];

        assert.deepStrictEqual(_joinContiguousGroups(groups), expectedResult);
        
        groups.reverse();
        assert.deepStrictEqual(_joinContiguousGroups(groups), expectedResult);
    });

    it("joins multiple touching groups", function() {
        const groups = [
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:03:00 PM")),
                presences: [0, 1]
            },
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:03:00 PM"),
                    new Date("1/1/2019 1:06:00 PM")),
                presences: [0, 1, 2]
            },
            {
                timeSpan: new TimeSpan(
                    new Date("1/1/2019 1:06:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")),
                presences: [0, 1]
            },
        ];

        assert.deepStrictEqual(
            _joinContiguousGroups(groups),
            [
                {
                    timeSpan: new TimeSpan(
                        new Date("1/1/2019 1:00:00 PM"),
                        new Date("1/1/2019 1:10:00 PM")),
                    presences: [0, 1, 2],
                }
            ]);
    });
});
