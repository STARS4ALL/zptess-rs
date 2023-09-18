use std::collections::HashMap;
use std::hash::Hash;

// statistcal::mode has a bug when there is no clear mode
// This implementation fixes that
pub fn mode<T>(v: &[T]) -> Option<T>
where
    T: Hash + Copy + Eq,
{
    match v.len() {
        0 => None,
        1 => Some(v[0]),
        _ => {
            let mut counter = HashMap::new();
            for x in v.iter() {
                counter.entry(x).and_modify(|c| *c += 1).or_insert(1);
            }
            let mut max = 1;
            let mut mode = None;
            for (val, count) in counter.iter() {
                if *count == max {
                    // Handle multimodal
                    mode = None;
                } else if *count > max {
                    max = *count;
                    mode = Some(**val);
                }
            }
            mode
        }
    }
}
