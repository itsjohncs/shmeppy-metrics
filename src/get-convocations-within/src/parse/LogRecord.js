export class LogRecord {
    constructor({requestId, datetime, message}) {
        this.requestId = requestId;
        this.datetime = datetime;
        this.message = message;
    }
}


/**
 * Extracts data from a log line.
 *
 * Only the new(ish) log format (as of Oct 2018) is supported. Returns `null`
 * if the log line isn't in a recognized format.
 */
export function makeLogRecord(line) {
    const re = /^([-\w]+) shmeppy-app: \(([^)]+)\) \[([A-Z]+) - ([0-9]{1,2}\/[0-9]{1,2}\/[0-9]{4}) ([0-9]{1,2}:[0-9]{1,2}:[0-9]{1,2} (?:AM|PM))\] (.+)$/;
    const match = re.exec(line);
    if (!match) {
        return null;
    }

    const [_, server, requestId, level, rawDate, rawTime, message] = match;
    const datetime = new Date(Date.parse(`${rawDate} ${rawTime} UTC`));
    return new LogRecord({
        requestId,
        datetime,
        message
    });
}
