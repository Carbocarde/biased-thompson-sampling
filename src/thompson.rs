use ordered_float::NotNan;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::os::raw::c_double;

extern "C" {
    /// the way I think of it is actually (a + 1, b + 1)
    fn boost_ibeta_inv(a: c_double, b: c_double, p: c_double) -> c_double;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThompsonInfo {
    pub interesting: u64,
    pub uninteresting: u64,
}

pub fn skew_percentile(
    sampled_point: NotNan<f64>,
    runtime: &Option<NotNan<f64>>,
    user_bias: &NotNan<f64>,
) -> NotNan<f64> {
    let mut time_scaler =
        NotNan::new(100.0).unwrap() / runtime.unwrap_or(NotNan::new(0.01).unwrap());

    // A script with bias of 5 is weighted to be equal to an equivalent script that runs 5x as fast.
    time_scaler *= user_bias;

    sampled_point * time_scaler
}

/// Prefer entries with low runtime.
/// Entries without runtimes will always be run first.
pub fn thompson_sampling_bias_runtime(
    entries: &[&ThompsonInfo],
    runtimes: &[&Option<NotNan<f64>>],
    user_biases: &[&NotNan<f64>],
) -> Option<usize> {
    // Handle uninitialized runtimes by assuming the fastest runtime.
    let mut selected_entry_index: Option<usize> = None;
    let mut selected_entry_percentile: NotNan<f64> = NotNan::new(-1.0).unwrap();
    for (index, entry) in entries.iter().enumerate() {
        let skewed_percentile = thompson_step_bias_runtime(
            entry.interesting,
            entry.uninteresting,
            runtimes[index],
            user_biases[index],
        );

        if skewed_percentile > selected_entry_percentile {
            selected_entry_index = Some(index);
            selected_entry_percentile = skewed_percentile;
        }
    }
    println!("Selected: {:?}", selected_entry_index);

    selected_entry_index
}

/// Returns a vector mapping the nth selected entry to its index.
///
/// Ex. [0, 2, 1]: The first element was ranked first, the third second, and second third.
pub fn thompson_ranking_bias_runtime(
    entries: &[&ThompsonInfo],
    runtimes: &[&Option<NotNan<f64>>],
    user_biases: &[&NotNan<f64>],
) -> Vec<usize> {
    let mut percentiles = vec![NotNan::new(0.0).unwrap(); entries.len()];
    for (index, entry) in entries.iter().enumerate() {
        percentiles[index] = thompson_step_bias_runtime(
            entry.interesting,
            entry.uninteresting,
            runtimes[index],
            user_biases[index],
        );
    }

    let mut sorted_percentile_index_mapping = percentiles.iter().enumerate().collect::<Vec<_>>();
    sorted_percentile_index_mapping.sort_by_key(|&(_, percentile)| percentile);

    sorted_percentile_index_mapping
        .iter()
        .rev()
        .map(|(index, _percentile)| *index)
        .collect()
}

fn thompson_step_bias_runtime(
    interesting: u64,
    uninteresting: u64,
    runtime: &Option<NotNan<f64>>,
    user_bias: &NotNan<f64>,
) -> NotNan<f64> {
    let mut rng = rand::thread_rng();
    // Random number from 0.0 to 1.0 inclusive
    let random_float = rng.gen_range(0.0..1.0);

    let percentile: f64;
    unsafe {
        percentile = boost_ibeta_inv(
            (interesting + 1) as f64,
            (uninteresting + 1) as f64,
            random_float,
        );
    }

    let skewed_percentile = skew_percentile(NotNan::new(percentile).unwrap(), runtime, user_bias);

    // println!(
    //     "Total percentage of area at point {:.4}: {:.2}% B({}, {}) Skewed area: {:.2}",
    //     random_float * 100.0,
    //     percentile,
    //     (uninteresting + 1) as f64,
    //     (interesting + 1) as f64,
    //     skewed_percentile
    // );

    skewed_percentile
}

pub fn thompson_sampling(entries: &[&ThompsonInfo], user_biases: &[&NotNan<f64>]) -> Option<usize> {
    let mut selected_entry_index: Option<usize> = None;
    let mut selected_entry_percentile: f64 = -1.0;
    for (index, entry) in entries.iter().enumerate() {
        let mut percentile = thompson_step(entry.interesting, entry.uninteresting);
        // println!(
        //     "Total percentage of area at point {:.4}: {:.2}%",
        //     percentile,
        //     random_float * 100.0
        // );
        percentile *= f64::from(*user_biases[index]);

        if percentile > selected_entry_percentile {
            selected_entry_index = Some(index);
            selected_entry_percentile = percentile
        }
    }
    selected_entry_index
}

/// Returns a vector mapping the nth selected entry to its index.
///
/// Ex. [0, 2, 1]: The first element was ranked first, the third second, and second third.
pub fn thompson_ranking(entries: &[&ThompsonInfo]) -> Vec<usize> {
    let mut percentiles = vec![0.0; entries.len()];
    for (idx, entry) in entries.iter().enumerate() {
        percentiles[idx] = thompson_step(entry.interesting, entry.uninteresting);
    }

    let mut sorted_percentile_index_mapping = percentiles
        .iter()
        .map(|x| NotNan::new(*x).unwrap())
        .enumerate()
        .collect::<Vec<_>>();
    sorted_percentile_index_mapping.sort_by_key(|&(_, percentile)| percentile);

    sorted_percentile_index_mapping
        .iter()
        .rev()
        .map(|(idx, _percentile)| *idx)
        .collect()
}

fn thompson_step(interesting: u64, uninteresting: u64) -> f64 {
    let mut rng = rand::thread_rng();
    // Random number from 0.0 to 1.0 inclusive
    let random_float: f64 = rng.gen_range(0.0..1.0);
    // println!("Percentile to sample: {}", random_float);
    let percentile: f64;
    unsafe {
        percentile = boost_ibeta_inv(
            (interesting + 1) as f64,
            (uninteresting + 1) as f64,
            random_float,
        );
    }
    // println!(
    //     "Total percentage of area at point {:.4}: {:.2}% B({}, {})",
    //     random_float * 100.0,
    //     percentile,
    //     (interesting + 1) as f64,
    //     (uninteresting + 1) as f64
    // );
    percentile
}

/// Returns the 50th percentile of the beta distribution.
pub fn dist_area_at_percentile(entry: &ThompsonInfo, area: f64) -> f64 {
    let point: f64;
    unsafe {
        point = boost_ibeta_inv(
            (entry.interesting + 1) as f64,
            (entry.uninteresting + 1) as f64,
            area,
        );
    }
    point
}

#[test]
fn test_thompson_sampling_none() {
    assert_eq!(thompson_sampling(&vec![], &vec![]), None);
}

#[test]
fn test_thompson_sampling_one() {
    assert_eq!(
        thompson_sampling(
            &[&ThompsonInfo {
                interesting: 0,
                uninteresting: 0
            }],
            &[&NotNan::new(1.0).unwrap(), &NotNan::new(1.0).unwrap()]
        ),
        Some(0)
    );
}

#[test]
fn test_thompson_sampling_prefer_interesting() {
    assert_eq!(
        thompson_sampling(
            &[
                &ThompsonInfo {
                    interesting: 0,
                    uninteresting: 100,
                },
                &ThompsonInfo {
                    interesting: 100,
                    uninteresting: 0
                }
            ],
            &[&NotNan::new(1.0).unwrap(), &NotNan::new(1.0).unwrap()]
        ),
        Some(1)
    );
}

#[test]
fn test_thompson_sampling_bias_prefer_fast() {
    assert_eq!(
        thompson_sampling_bias_runtime(
            &[
                &ThompsonInfo {
                    interesting: 100,
                    uninteresting: 100
                },
                &ThompsonInfo {
                    interesting: 100,
                    uninteresting: 100
                }
            ],
            &[
                &Some(NotNan::new(1.0).unwrap()),
                &Some(NotNan::new(100.0).unwrap())
            ],
            &[&NotNan::new(1.0).unwrap(), &NotNan::new(1.0).unwrap()]
        ),
        Some(0)
    );
}

#[test]
fn test_thompson_sampling_bias_prefer_unknown() {
    assert_eq!(
        thompson_sampling_bias_runtime(
            &[
                &ThompsonInfo {
                    interesting: 100,
                    uninteresting: 0
                },
                &ThompsonInfo {
                    interesting: 0,
                    uninteresting: 0
                }
            ],
            &[&Some(NotNan::new(1.0).unwrap()), &None],
            &[&NotNan::new(1.0).unwrap(), &NotNan::new(1.0).unwrap()]
        ),
        Some(1)
    );
}
