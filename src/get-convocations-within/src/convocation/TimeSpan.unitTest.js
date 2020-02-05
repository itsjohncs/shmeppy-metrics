import assert from "assert";

import TimeSpan from "./TimeSpan.js";


describe("TimeSpan", function() {
    describe("fill", function() {
        it("throws if given no args", function() {
            assert.throws(function() {TimeSpan.fill()});
        });

        it("is the identity function is given 1 arg", function() {
            const span = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));
            assert.deepStrictEqual(TimeSpan.fill(span), span);
        });

        it("returns timespan spanning the entire range", function() {
            const spans = [
                new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")),
                new TimeSpan(
                    new Date("1/1/2019 1:10:00 PM"),
                    new Date("1/1/2019 1:20:00 PM")),
            ];
            assert.deepStrictEqual(
                TimeSpan.fill(...spans),
                new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:20:00 PM")));
        });

        it("works with three args", function() {
            const spans = [
                new TimeSpan(
                    new Date("1/1/2019 1:00:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")),
                new TimeSpan(
                    new Date("1/1/2019 1:10:00 PM"),
                    new Date("1/1/2019 1:20:00 PM")),
                new TimeSpan(
                    new Date("1/1/2019 12:50:00 PM"),
                    new Date("1/1/2019 1:15:00 PM")),
            ];
            assert.deepStrictEqual(
                TimeSpan.fill(...spans),
                new TimeSpan(
                    new Date("1/1/2019 12:50:00 PM"),
                    new Date("1/1/2019 1:20:00 PM")));
        });
    });

    describe("intersect", function() {
        it("returns null if they don't intersect", function() {
            const a = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));
            const b = new TimeSpan(
                new Date("1/1/2019 1:20:00 PM"),
                new Date("1/1/2019 1:30:00 PM"));

            assert.strictEqual(TimeSpan.intersect(a, b), null);
        });

        it("returns intersection A partially contains B", function() {
            const a = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));
            const b = new TimeSpan(
                new Date("1/1/2019 1:05:00 PM"),
                new Date("1/1/2019 1:15:00 PM"));

            assert.deepStrictEqual(
                TimeSpan.intersect(a, b),
                new TimeSpan(
                    new Date("1/1/2019 1:05:00 PM"),
                    new Date("1/1/2019 1:10:00 PM")));
        });

        it("returns B if A completely contains B", function() {
            const a = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));
            const b = new TimeSpan(
                new Date("1/1/2019 1:01:00 PM"),
                new Date("1/1/2019 1:09:00 PM"));

            assert.deepStrictEqual(TimeSpan.intersect(a, b), b);
        });

        it("returns A if given (A, A)", function() {
            const a = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));

            assert.deepStrictEqual(TimeSpan.intersect(a, a), a);
        });
    });

    describe("overlapsWith", function() {
        it("returns false if not overlapping", function() {
            const a = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));
            const b = new TimeSpan(
                new Date("1/1/2019 1:20:00 PM"),
                new Date("1/1/2019 1:30:00 PM"));

            assert.strictEqual(a.isTouching(b), false);
        });

        it("returns true if overlapping", function() {
            const a = new TimeSpan(
                new Date("1/1/2019 1:00:00 PM"),
                new Date("1/1/2019 1:10:00 PM"));
            const b = new TimeSpan(
                new Date("1/1/2019 1:05:00 PM"),
                new Date("1/1/2019 1:30:00 PM"));

            assert.strictEqual(a.isTouching(b), true);
        });
    })
});
