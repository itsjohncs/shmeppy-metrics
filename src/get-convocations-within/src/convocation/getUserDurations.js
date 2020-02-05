import TimeSpan from "./TimeSpan.js";


function _addDuration(durations, accountId, milliseconds) {
    if (durations.has(accountId)) {
        durations.set(accountId, durations.get(accountId) + milliseconds);
    } else {
        durations.set(accountId, milliseconds);
    }
}


function getUserDurations({timeSpan, presences}) {
    const durations = new Map();
    for (const presence of presences) {
        const intersection = TimeSpan.intersect(presence.during, timeSpan);
        _addDuration(
            durations,
            presence.getAccountId(),
            intersection.getDuration());
    }

    return durations;
}


export default getUserDurations;
