import assert from "assert";

import _groupOverlappingPresences from "./_groupOverlappingPresences.js";
import _joinContiguousGroups from "./_joinContiguousGroups.js";
import _getActiveTimespans from "./_getActiveTimespans.js";
import getUserDurations from "./getUserDurations.js";
import UserPresence from "./UserPresence.js";
import TimeSpan from "./TimeSpan.js";


function _minutes(minutes) {
    return minutes * 1000 * 60;
}


function getConvocations({gameRequests}) {
    if (gameRequests.length === 0) {
        return [];
    }
    assert(gameRequests.every(r => r.gameId === gameRequests[0].gameId))

    // Go through all the gameRequests and get a set of "active timespans" when
    // the game had activity in it (in this case, operations were comitted).
    const activeTimespans = _getActiveTimespans(gameRequests);

    const presences = UserPresence.fromMany(gameRequests);

    // Someone needs to be present for at least 30 seconds for their presence
    // to count. This is pretty arbitrary and could exclude someone with a
    // *very* bad connection, but that's probably fine.
    const longEnoughPresences = presences.filter(
        i => i.during.getDuration() >= 30);

    const groups = _groupOverlappingPresences(longEnoughPresences);

    const cutUpGroups = [];
    for (const group of groups) {
        for (const activeTimespan of activeTimespans) {
            const intersection = TimeSpan.intersect(group.timeSpan, activeTimespan);
            if (intersection) {
                cutUpGroups.push({
                    timeSpan: intersection,
                    presences: [...group.presences],
                });
            }
        }
    }

    // If I just cut or shrink any groups that aren't within an active timespan
    // I think this call to _joinContiguousGroups will do any joining
    // necessary... If indeed any joining is ever necessary... hmm...
    const convocations = _joinContiguousGroups(cutUpGroups);

    return convocations.filter(function(convocation) {
        const userDurations = getUserDurations(convocation);
        let count = 0;
        for (const milliseconds of userDurations.values()) {
            if (milliseconds >= _minutes(45)) {
                ++count;
            }
        }
        return count >= 2;
    });
}


export default getConvocations;
