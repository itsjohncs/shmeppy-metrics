import assert from "assert";

import TimeSpan from "./TimeSpan.js";


function _getPivots(presences) {
    const pivots = new Set();
    for (const {during} of presences) {
        pivots.add(during.start.getTime());
        pivots.add(during.end.getTime());
    }

    return Array.from(pivots).sort().map(v => new Date(v));
}


function _presencesAt(presences, time) {
    return presences.filter(i => i.during.contains(time));
}


function _groupOverlappingPresences(presences) {
    // This algorithm assumes a zero-time request is impossible. If this
    // triggers (because a zero-time request really happened) I can just filter
    // out the request at some point above here in the call stack.
    assert(presences.every(i => i.during.getDuration() !== 0));

    const presencesByPivot = _getPivots(presences).map(function (pivot) {
        return {pivot, presences: _presencesAt(presences, pivot)};
    });

    // The last pivot should never have any presences matching it. This should
    // be implied by the first assertion.
    assert(
        presencesByPivot.length === 0 ||
        presencesByPivot[presencesByPivot.length - 1].presences.length === 0);

    // Because _getPivots returned the pivots in ascending order,
    // presencesByPivot is also sorted in the same way.
    const overlappingPresences = [];
    for (let i = 0; i < presencesByPivot.length - 1; ++i) {
        const start = presencesByPivot[i].pivot;
        const end = presencesByPivot[i + 1].pivot;
        const presences = presencesByPivot[i].presences;
        overlappingPresences.push({
            timeSpan: new TimeSpan(start, end),
            presences,
        });
    }

    return overlappingPresences.filter(({presences}) => presences.length >= 2);
}


export default _groupOverlappingPresences;
