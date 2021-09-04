//!
//! The invariant calculation.
//!

use crate::{N, ZERO};
use primitive_types::U256;

///
/// The `D` invariant calculation function.
///
/// The function is quite generic and does not work on token balances directly.
/// The only requirement for the `values` is to be of the same precision
/// to avoid incorrect amplification.
///
pub fn calculate(
    values: [U256; N],
    amplifier: u64,
) -> U256 {
    let mut sum = *ZERO;
    for i in 0..N {
        sum += values[i];
    }

    if sum != *ZERO {
        let mut d_prev = *ZERO;
        let mut d = sum;

        let amplifier_n: U256 = (amplifier * N as u64).into();

        for _n in 0..15 {
            if (d > d_prev && d - d_prev > *ZERO) ||
                (d <= d_prev && d_prev - d > *ZERO) { break; }
            let mut d_p = d;

            for i in 0..N {
                // +1 is to prevent division by 0
                d_p = d_p * d / (values[i] * U256::from(N) + 1);
            }

            d_prev = d;
            d = (amplifier_n * sum + d_p * U256::from(N)) * d /
                ((amplifier_n - 1) * d + U256::from(N+1) * d_p);
        }

        d
    } else {
        *ZERO
    }
}

#[test]
fn ok_zero_values() {
    let values = [0 as U256; N];
    let amplifier: u64 = 100;

    assert_eq!(calculate(values, amplifier), 0, "Invalid invariant");
}

#[test]
fn ok_some_values() {
    let values = [1_E6 as U256; N];
    let amplifier: u64 = 100;

    assert_eq!(calculate(values, amplifier), 2_E6, "Invalid invariant");
}

#[test]
fn ok_some_values_amplified_swap() {
    let amplifier: u64 = 100;

    let direct = calculate([1_E6 as U256, 1_E18 as U256], amplifier);
    let reverse = calculate([1_E18 as U256, 1_E6 as U256], amplifier);

    assert!(
        direct > reverse && direct - reverse < U256::from(10).pow(U256::from(18)) ||
            reverse > direct && reverse - direct < U256::from(10).pow(U256::from(18)),
        "Invariant depends on the value order for some reason");
}