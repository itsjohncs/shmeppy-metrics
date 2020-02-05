import assert from "assert";
import readline from "readline";

import extractGameRequests from "./parse/extractGameRequests.js";
import getConvocations from "./convocation/getConvocations.js";
import groupBy from "./util/groupBy.js";
import getUserDurations from "./convocation/getUserDurations.js";


function _getLines(file) {
    return new Promise(function(resolve, reject) {
        const input = readline.createInterface({
            input: file,
            output: null,
            terminal: false,
            crlfDelay: Infinity,
        });

        const lines = [];
        input.on("line", function(line) {
            lines.push(line);
        });

        input.on("close", function() {
            resolve(lines);
        });
    });
}


function* allRequestsInConvocation(convocation) {
    for (const presence of convocation.presences) {
        for (const request of presence.requests) {
            yield request;
        }
    }
}


function getConvocationGameId(convocation) {
    return convocation.presences[0].getGameId();
}


async function main() {
    if (!process.argv[2]) {
        console.error(`get-convocations-within YYYY-MM-DD`);
        return;
    }
    const start = new Date(process.argv[2])
    const end = new Date(start.getTime() + (24 * 60 * 60 * 1000));

    const allGameRequests = extractGameRequests(
        await _getLines(process.stdin));

    const gameRequestsByGameId = groupBy(allGameRequests, "gameId");
    const results = [];
    for (const gameRequests of gameRequestsByGameId.values()) {
        for (const convocation of getConvocations({gameRequests})) {
            // If convocations last more than 24 hours, the caching strategy
            // that build-convocations-aggregate uses won't work.
            assert(
                (convocation.timeSpan.end.getTime() -
                 convocation.timeSpan.start.getTime()) / (60 * 60 * 1000) < 24)

            if (!(start <= convocation.timeSpan.start &&
                    convocation.timeSpan.start < end)) {
                continue;
            }

            const admins = new Set();
            const players = new Set();
            for (const request of allRequestsInConvocation(convocation)) {
                if (request.isAdmin) {
                    admins.add(request.accountId);
                } else {
                    players.add(request.accountId);
                }
            }

            results.push({
                admins: Array.from(admins),
                players: Array.from(players),
                gameId: getConvocationGameId(convocation),
                start: convocation.timeSpan.start.toISOString(),
                end: convocation.timeSpan.end.toISOString(),
            });
        }
    }

    console.log(JSON.stringify(results));
}

main();
