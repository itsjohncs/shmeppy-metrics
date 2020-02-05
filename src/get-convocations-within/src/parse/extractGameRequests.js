import {makeLogRecord} from "./LogRecord.js";
import {makeGameRequest} from "./GameRequest.js";
import groupBy from "../util/groupBy.js";


function extractGameRequests(lines) {
    const allRecords = lines.map(makeLogRecord).filter(i => i !== null);
    const recordsByRequestId = groupBy(allRecords, "requestId");
    const gameRequests = (
        Array.from(recordsByRequestId.values())
             .map(makeGameRequest)
             .filter(i => i !== null));
    return gameRequests;
}


export default extractGameRequests;
