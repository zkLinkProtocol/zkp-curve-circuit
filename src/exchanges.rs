//!
//! The swap consequences calculation.
//!

use crate::{N, Balance, PRECISION_MUL, ZERO};
use primitive_types::U256;

/// The token being withdrawn balance get_y the swap.
pub fn get_y(
    mut balances: [Balance; N],
    amplifier: u64,
    d: U256,
    token_x_idx: usize,
    token_y_idx: Option<usize>,
    after_x: Option<Balance>,
    is_get_y_d: bool
) -> Balance {
    // Calculate x[j] if one makes x[i] = x
    //
    // Done by solving quadratic equation iteratively.
    // x_1**2 + x_1 * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
    // x_1**2 + b*x_1 = c
    //
    // x_1 = (x_1**2 + c) / (2*x_1 + b)

    assert!((is_get_y_d && token_y_idx.is_none()) || (!is_get_y_d && token_y_idx.is_some()));
    assert_ne!(token_x_idx, token_y_idx.unwrap(), "Cannot exchange between the same coins");
    assert!(token_x_idx < N, "There is no x token Id in the pool");
    assert!(token_y_idx.unwrap() < N, "There is no y token Id in the pool");

    balances.iter_mut().for_each(|balance| *balance *= PRECISION_MUL[0]);

    let an: U256 = (amplifier * N as u64).into();

    // let x_magnitude_diff = tokens[token_x_idx].magnitude_diff() * PRECISION_MUL;
    // let y_magnitude_diff = tokens[token_y_idx].magnitude_diff() * PRECISION_MUL;

    let mut c = d;
    let mut s= *ZERO;
    // let after_x_p = after_x * x_magnitude_diff;

    for i in 0..N {
        let after_x_p = if is_get_y_d {
            if i != token_x_idx {
                balances[i]
            } else { continue; }
        } else {
            if i == token_x_idx {
                after_x.unwrap()
            } else if i != token_y_idx.unwrap() {
                balances[i]
            } else { continue; }
        };
        s += after_x_p;
        c = c * d / (after_x_p * N)
    }

    c = c * d / (an * N);

    let b: Balance = s + d / an;
    let mut y = d;
    for _ in 0..255{
        let y_next = (y * y + c) / (U256::from(2) * y + b - d);

        if (y > y_next && y - y_next > U256::one())
            || (y <= y_next && y_next - y > U256::one()) {
            return y_next;
        }
        y = y_next;
    }

    y
}

#[test]
fn ok_equal_precision() {
    let balances = [U256::from(1_000);N];
    let amp = 100;
    let new_y = get_y(
        balances,
        amp,
        crate::calculate(balances, amp),
        0,
        Some(1),
        1_050.into(),
        false,
    );

    assert_eq!(new_y, 950, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_equal_precision_amplified() {
    let balances = [U256::from(1_000_000);N];
    let amp = 100;
    let new_y = get_y(
        balances,
        amp,
        crate::calculate(balances, amp),
        0,
        Some(1),
        Some(U256::from(1_900_000)),
        false,
    );

    assert_eq!(new_y, 130_370, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_lesser_bigger() {
    let balances = [U256::from(1_000_000), U256::from(10).pow(18.into())];
    let amp = 100;
    let new_y = get_y(
        [1_E6 as Balance, 1_E18 as Balance],
        amp,
        crate::calculate(balances,amp),
        0,
        Some(1),
        Some(U256::from(1_050_000)),
        false,
    );

    assert_eq!(new_y, 950_024_800_946_586_013, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_lesser_bigger_amplified() {
    let balances = [U256::from(1_000_000), U256::from(10).pow(18.into())];
    let amp = 100;
    let new_y = get_y(
        balances,
        amp,
        crate::calculate(balances,amp),
        0,
        Some(1),
        Some(U256::from(1_950_000)),
        false,
    );

    assert_eq!(new_y, 94_351_900_636_131_207, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_bigger_lesser() {
    let balances = [U256::from(10).pow(18.into()),U256::from(1_000_000)];
    let amp = 100;
    let new_y = get_y(
        balances,
        amp,
        crate::calculate(balances,amp),
        0,
        Some(1),
        Some(U256::from(1_050) * U256::from(10).pow(15.into())),
        false,
    );

    assert_eq!(new_y, 950_024, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_bigger_lesser_amplified() {
    let balances = [
        U256::from(10).pow(18.into()),
        U256::from(10).pow(6.into()),
    ];
    let amp = 100;
    let new_y = get_y(
        balances,
        amp,
        crate::calculate(balances, amp),
        0,
        Some(1),
        Some(U256::from(1.950)),
        false,
    );

    assert_eq!(new_y, 94_351, "The balance get_y withdrawal does not match the reference");
}

#[test]
#[should_panic]
fn error_same_tokens() {
    let balances = [U256::from(1000);N];
    let amp = 100;
    get_y(
        balances,
        amp,
        crate::calculate(balances, amp),
        1,
        Some(1),
        Some(Balance::from(100)),
        false,
    );
}