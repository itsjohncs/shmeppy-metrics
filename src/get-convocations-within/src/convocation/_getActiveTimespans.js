import assert from "assert";

import TimeSpan from "./TimeSpan.js";


const COMMITTED_RE = /^Committed [0-9]+ operation\(s\).$/;

// The number of minutes a single committed operation will make a game active
// for. The window will be centered on the operation committal time.
export const _ACTIVITY_WINDOW_MINUTES = 60;


export function _addMinutes(datetime, minutes) {
	return new Date(datetime.getTime() + minutes * 1000 * 60);
}


function _recordToActiveTimespan(record) {
	if (COMMITTED_RE.exec(record.message)) {
		return new TimeSpan(
			_addMinutes(record.datetime, -(_ACTIVITY_WINDOW_MINUTES / 2)),
			_addMinutes(record.datetime, (_ACTIVITY_WINDOW_MINUTES / 2)));
	}

    return undefined;
}


function _sortedTimespans(timespans) {
    return [...timespans].sort(
        function({start: a}, {start: b}) {
            if (a > b) {
                return 1;
            } else if (a < b) {
                return -1;
            }

            return 0;
        });
}


function _cloneManyTimespans(timespans) {
	return timespans.map(
        timespan => new TimeSpan(timespan.start, timespan.end));
}


function _combineContiguousTimespans(unsortedTimespans) {
    if (unsortedTimespans.length === 0) {
        return [];
    }

    const timespans = _cloneManyTimespans(
    	_sortedTimespans(unsortedTimespans));
    const combinedTimespans = [timespans[0]];
    for (let i = 1; i < timespans.length; ++i) {
        // The timespan we will consider extending
        const base = combinedTimespans[combinedTimespans.length - 1];

        // The timespan we will consider extending base with
        const newcomer = timespans[i];

        if (newcomer.isTouching(base)) {
            // Extend the base group
            combinedTimespans[combinedTimespans.length - 1] = (
                TimeSpan.joinTouching(newcomer, base));
        } else {
            // Create a new base group
            combinedTimespans.push(newcomer);
        }
    }

    return combinedTimespans;
}


function _getActiveTimespans(gameRequests) {
	const activeTimespans = [];

	// Collect timespans without worrying about overlap
	for (const request of gameRequests) {
		for (const record of request.records) {
			const activeTimespan = _recordToActiveTimespan(record);
			if (activeTimespan) {
				activeTimespans.push(activeTimespan);
			}
		}
	}

	// Join any contiguous timespans
    return _combineContiguousTimespans(activeTimespans);
}


export default _getActiveTimespans;
