function _appendInMap(map, key, value) {
    const array = map.get(key);
    if (array) {
        array.push(value);
    } else {
        map.set(key, [value]);
    }
}


function groupBy(lst, key) {
    const result = new Map();
    for (const i of lst) {
        _appendInMap(result, i[key], i);
    }

    return result;
}


export default groupBy;
