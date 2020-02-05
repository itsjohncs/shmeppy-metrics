import assert from "assert";


function _sortEntries(entries) {
	return Array.from(entries).sort(function([k1], [k2]) {
		if (k1 < k2) {
			return -1;
		} else if (k1 > k2) {
			return 1;
		}

		return 0;
	});
}


function assertMapsAreEqual(a, b) {
	assert.deepStrictEqual(
		_sortEntries(a.entries()),
		_sortEntries(b.entries()));
}


export default assertMapsAreEqual;
