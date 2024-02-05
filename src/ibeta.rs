extern crate test;
#[cfg(test)]
use std::ffi::c_double;
#[cfg(test)]
use test::{black_box, Bencher};

#[cfg(test)]
extern "C" {
    /// the way you think of it is actually (a + 1, b + 1)
    fn boost_ibeta_inv(a: c_double, b: c_double, p: c_double) -> c_double;
    fn boost_ibeta(a: c_double, b: c_double, p: c_double) -> c_double;
}

#[test]
fn beta_inverse() {
    let a = 2;
    let b = 1;
    let p = 0.5;

    let result: f64;
    unsafe {
        result = boost_ibeta_inv(a as f64, b as f64, p);
    }

    println!(
        "Total percentage of area at point {}: {:.2}%",
        result,
        p * 100.0
    );

    assert_eq!(result, 0.7071067811865476);
}

#[test]
fn beta_inverse_half() {
    let a = 1000;
    let b = 1000;
    let p = 0.5;

    let result: f64;
    unsafe {
        result = boost_ibeta_inv((a + 1) as f64, (b + 1) as f64, p);
    }

    println!(
        "Total percentage of area at point {}: {:.2}%",
        result * 100.0,
        p * 100.0
    );

    let inv: f64;
    unsafe {
        inv = boost_ibeta((a + 1) as f64, (b + 1) as f64, result);
    }

    println!("Inverse {}: {:.2}%", result * 100.0, inv * 100.0);

    assert_eq!(result, 0.5);
}

#[bench]
fn basic_benchmark(ben: &mut Bencher) {
    let a = 2.0;
    let b = 1.0;
    let p = 0.5;

    ben.iter(|| {
        let result: f64;
        unsafe {
            result = boost_ibeta_inv(a, b, p);
        }
        black_box(result);
    });
}
