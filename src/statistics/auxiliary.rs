use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use tracing::warn;

pub fn round(x: f32, decimals: u32) -> f32 {
    let y = 10u32.pow(decimals) as f32;
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

pub fn magntude(freq: f32, freq_offset: f32, zp: f32) -> f32 {
    zp - 2.5 * (freq - freq_offset).log10()
}

pub fn mode_or_median(v: &[f32], precision: u32, label: &str) -> f32 {
    let v1: Vec<i32> = v
        .iter()
        .map(|x| (*x * (10u32.pow(precision)) as f32).round() as i32)
        .collect();
    if let Some(mode) = mode(&v1) {
        mode as f32 / (10u32.pow(precision) as f32)
    } else {
        warn!(
            "Mode for {} does not exists, calculating median instead",
            label
        );
        statistical::median(v)
    }
}
