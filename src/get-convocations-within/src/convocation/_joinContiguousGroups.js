import assert from "assert";

import TimeSpan from "./TimeSpan.js";


function minutes(n) {return 1000 * 60 * n;}
const ACCEPTABLE_BREAK_MS = minutes(60);


function _sortedGroups(unsortedGroups) {
    return [...unsortedGroups].sort(
        function({timeSpan: {start: a}}, {timeSpan: {start: b}}) {
            if (a > b) {
                return 1;
            } else if (a < b) {
                return -1;
            }

            return 0;
        });
}


function _uniq(lst) {
    return Array.from(new Set(lst));
}


function _joined(groupA, groupB) {
    return {
        timeSpan: TimeSpan.fill(groupA.timeSpan, groupB.timeSpan),
        presences: _uniq([...groupA.presences, ...groupB.presences]),
    };
}


function _joinContiguousGroups(unsortedGroups) {
    if (unsortedGroups.length === 0) {
        return [];
    }

    const groups = _sortedGroups(unsortedGroups);
    const joinedGroups = [{...groups[0]}];
    for (let i = 1; i < groups.length; ++i) {
        // The group we will consider extending
        const base = joinedGroups[joinedGroups.length - 1];

        // The group we will consider extending base with
        const group = groups[i];

        assert(base.timeSpan.end <= group.timeSpan.start);
        if (group.timeSpan.start - base.timeSpan.end < ACCEPTABLE_BREAK_MS) {
            // Extend the base group
            joinedGroups[joinedGroups.length - 1] = _joined(base, group);
        } else {
            // Create a new base group
            joinedGroups.push(group);
        }
    }

    return joinedGroups;
}


export default _joinContiguousGroups;
