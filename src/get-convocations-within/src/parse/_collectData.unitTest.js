import assert from "assert";

import _collectData from "./_collectData.js";
import assertMapsAreEqual from "../testutil/assertMapsAreEqual.js";


describe("_collectData", function() {
    // A weird edge case... We just do something halfway reasonable.
    it("returns an empty map when run against an empty list", function() {
        const actual = _collectData([], []);
        assert.deepStrictEqual(actual, new Map());
    });

    it("returns null if not all mappers give values", function() {
        const actual = _collectData([["name", ({name}) => name]],
                                    [{x: "y"}]);
        assert.strictEqual(actual, null);
    })

    it("collects unique results normally", function() {
        const actual = _collectData([["name", ({name}) => name]],
                                    [{name: "John"}]);
        assertMapsAreEqual(actual, new Map([["name", "John"]]));
    });

    it("errors if map functions returns value for multiple objects", function() {
        assert.throws(function() {
            _collectData([["name", ({name}) => name]],
                         [{name: "John"}, {name: "Bob"}]);
        });
    });

    it("collects multiple results", function() {
        const actual = _collectData([
            ["name", ({name}) => name],
            ["age", ({age}) => age],
        ], [{name: "John"}, {age: 26}]);
        assertMapsAreEqual(actual, new Map([["name", "John"], ["age", 26]]));
    });
});
