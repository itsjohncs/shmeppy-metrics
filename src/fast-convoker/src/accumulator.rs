use std::collections::BTreeMap;


pub fn push_onto_accumulator<Key: Ord + Copy, Value>(
        accumulator: &mut BTreeMap<Key, Vec<Value>>,
        key: Key,
        value: Value) {
    let lst = if let Some(lst) = accumulator.get_mut(&key) {
        lst
    } else {
        accumulator.insert(key, Vec::new());
        accumulator.get_mut(&key).unwrap()
    };

    lst.push(value)
}
