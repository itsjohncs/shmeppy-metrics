import assert from "assert";

import _collectData from "./_collectData.js";


const WEBSOCKET_OPENED_RE =
    /^Started websocket: GET \/game-socket\/([0-9]+) 1.1$/;
const CLIENT_ADDED_RE = /^Client added to client DB: (.+)$/;
const WEBSOCKET_CLOSED_RE = /^Finished websocket: [0-9]+ '.*'$/;


function _getWebsocketOpenedDatetime(record) {
    if (WEBSOCKET_OPENED_RE.exec(record.message)) {
        return record.datetime;
    }

    return undefined;
}


function _getGameId(record) {
    const match = WEBSOCKET_OPENED_RE.exec(record.message);
    if (match) {
        return parseInt(match[1], 10);
    }

    return undefined;
}


function _getAccountId(record) {
    const match = CLIENT_ADDED_RE.exec(record.message);
    if (match && match[1].startsWith("{ gameId")) {
        const accountIdMatch = /{ accountId: '([a-f0-9]+)'/.exec(match[1]);
        if (!accountIdMatch) {
            throw new Error(
                `Failed parsing account ID. \n\n${match[1]}`);
        }

        return accountIdMatch[1];
    } else if (match) {
        try {
            return JSON.parse(match[1]).account.accountId;
        } catch(err) {
            throw new Error(
                `Failed parsing account ID. ${err}\n\n${match[1]}`);
        }
    }

    return undefined;
}

function _getIsAdmin(record) {
    const match = CLIENT_ADDED_RE.exec(record.message);
    if (match && match[1].startsWith("{ gameId")) {
        const isAdminMatch = /#012  isAdmin: (true|false),/.exec(match[1]);
        if (!isAdminMatch) {
            throw new Error(
                `Failed parsing isAdmin. \n\n${match[1]}`);
        }

        return isAdminMatch[1] == "true";
    } else if (match) {
        try {
            return JSON.parse(match[1]).isAdmin;
        } catch(err) {
            throw new Error(
                `Failed parsing isAdmin. ${err}\n\n${match[1]}`);
        }
    }

    return undefined;
}


function _getWebsocketClosedDatetime(record) {
    if (WEBSOCKET_CLOSED_RE.exec(record.message)) {
        return record.datetime;
    }

    return undefined;
}


export class GameRequest {
    constructor({websocketOpenedDatetime,
                 websocketClosedDatetime,
                 gameId,
                 accountId,
                 isAdmin,
                 requestId,
                 records}) {
        this.requestId = requestId;
        this.websocketOpenedDatetime = websocketOpenedDatetime;
        this.websocketClosedDatetime = websocketClosedDatetime;
        this.gameId = gameId;
        this.isAdmin = isAdmin;
        this.accountId = accountId;
        this.records = records;
    }
}


function _mapToObject(map) {
    const result = {};
    for (const [k, v] of map.entries()) {
        result[k] = v;
    }

    return result;
}


export function makeGameRequest(records) {
    assert(
        records.length === 0 ||
        records.every(({requestId}) => requestId === records[0].requestId));

    const data = _collectData([
        ["websocketOpenedDatetime", _getWebsocketOpenedDatetime],
        ["websocketClosedDatetime", _getWebsocketClosedDatetime],
        ["gameId", _getGameId],
        ["accountId", _getAccountId],
        ["isAdmin", _getIsAdmin],
    ], records);
    if (!data) {
        return null;
    }

    return new GameRequest({
        ..._mapToObject(data),
        requestId: records[0].requestId,
        records,
    });
}
