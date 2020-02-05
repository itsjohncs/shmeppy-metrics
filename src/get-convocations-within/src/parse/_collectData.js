/**
 * Runs a set of mapping functions on `lst`, returning the results.
 *
 * `keysToMappers` should be a list of pairs like:
 *
 *      [
 *          ["name", ({name}) => name],
 *          ["age", ({age}) => age],
 *      ]
 *
 * `_collectData(keysToMappers, [{"name": "John"}, {"age": 26}])` would then
 * give the Map (not Object) `{"name": "John", "age": 26}`.
 *
 * A value must be provided by every mapper, otherwise `null` is returned. So
 * for example, `_collectData(keysToMappers, [{"name": "John"}])` would give
 * `null`.
 *
 * `_collectData` will throw an error if a mapper returns a value for more than
 * one item in `lst`.
 */
function _collectData(keysToMappers, lst) {
    const result = new Map();
    for (const item of lst) {
        for (const [k, mapper] of keysToMappers) {
            const mapped = mapper(item);
            if (mapped !== undefined) {
                if (result.has(k)) {
                    throw new Error("Conflict");
                } else {
                    result.set(k, mapped);
                }
            }
        }
    }

    if (result.size !== keysToMappers.length) {
        return null;
    }

    return result;
}


export default _collectData;
