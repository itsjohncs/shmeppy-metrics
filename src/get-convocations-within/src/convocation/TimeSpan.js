import assert from "assert";


function _min(...items) {
    return items.reduce((a, b) => a < b ? a : b);
}

function _max(...items) {
    return items.reduce((a, b) => a > b ? a : b);
}


class TimeSpan {
    constructor(start, end) {
        assert(end >= start, `end (${end}) must be after start (${start}).`);
        this.start = start;
        this.end = end;
    }

    static fill(...spans) {
        // Reduce, rather than Math.(min/max), is used because the latter will
        // cast to numbers.
        const start = _min(...spans.map(({start}) => start));
        const end = _max(...spans.map(({end}) => end));
        return new TimeSpan(start, end);
    }

    static intersect(a, b) {
        const start = _max(a.start, b.start);
        const end = _min(a.end, b.end);
        if (start < end) {
            return new TimeSpan(start, end);
        }

        return null;
    }

    static joinTouching(a, b) {
        if (!a.isTouching(b)) {
            throw new Error(`Expected a to touch b. ${a} ${b}`);
        }

        return new TimeSpan(_min(a.start, b.start), _max(a.end, b.end));
    }

    getDuration() {
        return this.end - this.start;
    }

    /**
     * True if overlapping other.
     *
     * Note that we return False if they only share an edge.
     */
    isTouching(other) {
        return this.start < other.end && other.start < this.end;
    }

    contains(time) {
        return this.start <= time && time < this.end;
    }
}


export default TimeSpan;
