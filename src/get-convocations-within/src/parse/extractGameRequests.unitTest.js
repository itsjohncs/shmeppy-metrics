import assert from "assert";

import extractGameRequests from "./extractGameRequests.js";


const SAMPLE_DATA = `shmeppy-0 shmeppy-app: (0b013fe0-2064-4bb9-ad37-652349f91628) [INFO - 5/17/2019 12:54:33 PM] Started websocket: GET /game-socket/1731281016 1.1
shmeppy-0 shmeppy-app: (0b013fe0-2064-4bb9-ad37-652349f91628) [INFO - 5/17/2019 12:54:33 PM] Client added to client DB: {"gameId":1731281016,"clientId":2722,"account":{"accountId":"d058fb863601d419","displayName":"John","email":"john@gmail.invalid"},"isAdmin":true,"lastPing":1580779727259}
shmeppy-0 shmeppy-app: (0b013fe0-2064-4bb9-ad37-652349f91628) [INFO - 5/17/2019 12:54:47 PM] Finished websocket: 1005 ''
shmeppy-0 shmeppy-app: (3a604ab1-c69e-4159-9f8d-234571dbbf41) [INFO - 5/17/2019 12:54:47 PM] Started websocket: GET /game-socket/1391676696 1.1
shmeppy-0 shmeppy-app: (3a604ab1-c69e-4159-9f8d-234571dbbf41) [INFO - 5/17/2019 12:54:47 PM] Client added to client DB: {"gameId":1391676696,"clientId":2722,"account":{"accountId":"d058fb863601d419","displayName":"John","email":"john@gmail.invalid"},"isAdmin":true,"lastPing":1580779727259}
shmeppy-0 shmeppy-app: (3a604ab1-c69e-4159-9f8d-234571dbbf41) [INFO - 5/17/2019 12:54:50 PM] Finished websocket: 1005 ''
shmeppy-0 shmeppy-app: (698fa66f-9473-4d4b-9118-428dd30d612e) [INFO - 5/17/2019 12:54:50 PM] Started websocket: GET /game-socket/1731281016 1.1
shmeppy-0 shmeppy-app: (698fa66f-9473-4d4b-9118-428dd30d612e) [INFO - 5/17/2019 12:54:50 PM] Client added to client DB: {"gameId":1731281016,"clientId":2723,"account":{"accountId":"d058fb863601d419","displayName":"John","email":"john@gmail.invalid"},"isAdmin":true,"lastPing":1580779727259}
shmeppy-0 shmeppy-app: (698fa66f-9473-4d4b-9118-428dd30d612e) [INFO - 5/17/2019 12:54:54 PM] Finished websocket: 1001 ''
`;


describe("extractGameRequests", function() {
	it("extracts the correct number of requests", function() {
		// Not a very detailed test. Mainly smoke.
		const gameRequests = extractGameRequests(SAMPLE_DATA.split("\n"));
		assert.strictEqual(gameRequests.length, 3);
		assert(gameRequests.every(
			request => request.accountId === "d058fb863601d419"));
	});
});
