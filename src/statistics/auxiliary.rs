use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

pub fn round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}

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
                match count.cmp(&max) {
                    Ordering::Equal => {
                        mode = None;
                    }
                    Ordering::Greater => {
                        max = *count;
                        mode = Some(**val);
                    }
                    _ => {}
                }
            }
            mode
        }
    }
}
