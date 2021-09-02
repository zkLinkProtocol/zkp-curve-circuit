pub mod exchanges;
pub mod invariant;

use primitive_types::{U256,H160};
use crate::invariant::calculate;
use crate::exchanges::get_y;

// The ETH address in the unsigned integer form.
pub type Balance = U256;

//The biggest integer type available to store balances
pub type Address = H160;
pub const ZERO: Balance = U256::from(0);
// The default computation precision.
// Balances are multiplied by it during invariant computation,
// to avoid division of integers of the same order of magnitude.
pub const PRECISION_MUL: [U256; N] = [U256::from(1_000_000);N];
// The maximum known token precision.
pub const MAX_TOKEN_PRECISION: u8 = 18;

// These constants must be set prior to compiling

// The number of tokens being traded in the pool.
const N: usize = 2;
const RATES:[U256;N] = [U256::default();N];

// fixed constants
const FEE_DENOMINATOR:U256 = U256::from(10).pow(10.into());
const PRECISION:U256 = U256::from(10).pow(18.into());

const ADMIN_FEE:U256 = U256::from(10).pow(10.into());
const FEE:U256 = U256::from(10).pow(10.into());

struct StableSwap{
    // The tokens being traded in the pool.
    tokens: [Address; N],
    // The Curve amplifier.
    amplifier: u64,
    // The balances of tokens
    balances: [Balance; N],
    // Total supply liquidity in the pool.
    total_supply: U256
}

// The Curve StableSwap.
impl StableSwap {

    // The contract constructor.
    pub fn new(
        tokens: [Address; N],
        amplifier: u64,
        balances: [Balance; N]
    ) -> Self {
        assert!(amplifier > 0, "The Curve amplifier cannot be zero");

        Self {
            tokens,
            amplifier,
            balances,
            total_supply: Default::default()
        }
    }

    fn xp(&self) -> [U256;N]{
        let mut res = RATES;
        res.iter_mut().enumerate().for_each(|(i,res)|*res= res* self.balances[i]/PRECISION);
        res
    }

    fn get_current_d(&self) -> U256{
        calculate(self.balances, self.amplifier)
    }

    /// Adds liquidity to the contract balances.
    pub fn add_liquidity(&mut self, amount:[U256;N], min_lp_quantity: [U256;N]) -> U256 {
        let amp = self.amplifier;
        let old_balances = self.balances;

        let d0 = self.get_current_d();
        let total_supply = self.total_supply;
        let mut new_balances = old_balances;
        for i in 0..N {
            if total_supply == 0 { assert!(amount[i] > 0)}
            new_balances[i] += amount[i];
        }

        let d1 = calculate(new_balances, amp);
        assert!(d1 > d0);

        let mut d2 = d1;
        let mut fees = [Balance::default();N];
        let mut lp_quantity = Balance::default();
        if total_supply > U256::zero(){
            let fee = FEE * N / (4 * (N - 1));
            for (i, new_balance) in new_balances.iter_mut().enumerate() {
                let ideal_balance = d1 *old_balances[i] / d0;
                let difference = if ideal_balance > new_balance{
                    ideal_balance - new_balance
                } else {
                    new_balance - ideal_balance
                };
                fees[i] = fee * difference / FEE_DENOMINATOR;
                self.balances[i] = new_balance - (fees[i] * admin_fee / FEE_DENOMINATOR);
                new_balance[i] -= fees[i];
            }
            d2 = calculate(new_balances, amp);
            lp_quantity = total_supply * (d2 - d0) / d0;
        } else {
            self.balances = new_balances;
            lp_quantity = d1;
        }
        assert!(mint_amount >= min_lp_quantity, "Slippage screwed you");

        lp_quantity
    }

    /// Removes liquidity to the contract balances.
    pub fn remove_liquidity(&mut self, amount: U256, min_amounts: [U256;N]) -> [U256;N]{
        let total_supply = self.total_supply;
        let mut amounts = [Balance::default();N];

        for (i, balance) in self.balances.iter_mut().enumerate() {
            let value = *balance * amount / total_supply;
            assert!(value >= min_amounts[i], "Withdrawal resulted in fewer coins than expected");
            *balance = balance - value;
            amounts[i] = value;
        }
        amounts

    }

    /// Removes liquidity to the contract balances and withdraw one coins.
    pub fn remove_liquidity_one_coin(&mut self, amount: U256, token_index: usize, min_amount: U256) -> U256{
        let (dy, dy_fee, _total_supply) = self.calc_withdraw_coin_lp_by_removing_lp(amount, token_index);
        assert!(dy >= min_amount, "Not enough coins removed");

        self.balances[i] -= (dy + dy_fee * ADMIN_FEE / FEE_DENOMINATOR);
        dy

    }

    /// Exchanges the tokens, consuming some of the `zksync::msg.token_address` and returning
    /// some of the `withdraw_token_address`.
    pub fn swap(
        &mut self,
        withdraw_address: Address,
        withdraw_token_address: Address,
        min_withdraw: Balance,
    ) {

        let deposit_idx = self.token_position(Address::from_address(zksync::msg.token_address));
        let withdraw_idx = self.token_position(withdraw_token_address);

        let balance_array = self.balances;

        assert_ne!(balance_array[deposit_idx], 0, "Deposit token balance is zero");
        assert_ne!(balance_array[withdraw_idx], 0, "Withdraw token balance is zero");

        let new_x = balance_array[deposit_idx] + zksync::msg.amount;
        let new_y = exchanges::get_y(
            balance_array,
            self.amplifier,
            calculate(balance_array, self.amplifier),
            deposit_idx,
            Some(withdraw_idx),
            Some(new_x),
            false,
        );

        let old_y = balance_array[withdraw_idx];
        assert!(old_y >= min_withdraw + new_y,
            "Exchange resulted in fewer coins than expected");
        let withdraw_amount = old_y - new_y;

        self.transfer(
            withdraw_address,
            withdraw_token_address,
            withdraw_amount,
        );
    }

    ///
    /// Given the amount to withdraw, returns the amount that must be deposited.
    ///
    pub fn get_dx(
        &self,
        deposit_token_address: Address,
        withdraw_token_address: Address,
        to_withdraw: Balance,
    ) -> Balance {
        let deposit_idx = self.token_position(deposit_token_address);
        let withdraw_idx = self.token_position(withdraw_token_address);

        assert_ne!(self.balances[deposit_idx], 0, "Deposit token balance is zero");
        assert_ne!(self.balances[withdraw_idx], 0, "Withdraw token balance is zero");

        let after_withdrawal = self.balances[withdraw_idx] - to_withdraw;

        let after_deposit = exchanges::get_y(
            balance_array,
            self.amplifier,
            calculate(balance_array, self.amplifier),
            withdraw_idx,
            Some(deposit_idx),
            Some(after_withdrawal),
            false,
        );

        after_deposit - balance_array[deposit_idx]
    }

    ///
    /// Given the amount to deposit, returns the amount that will be withdrawn.
    ///
    pub fn get_dy(
        &self,
        deposit_token_address: Address,
        withdraw_token_address: Address,
        to_deposit: Balance,
    ) -> Balance {
        let deposit_idx = self.token_position(deposit_token_address);
        let withdraw_idx = self.token_position(withdraw_token_address);

        assert_ne!(balance_array[deposit_idx], 0, "Deposit token balance is zero");
        assert_ne!(balance_array[withdraw_idx], 0, "Withdraw token balance is zero");

        let after_deposit = self.balances[deposit_idx] + to_deposit;

        let after_withdrawal = exchanges::get_y(
            self.balances,
            self.amplifier,
            calculate(self.balances, self.amplifier),
            deposit_idx,
            Some(withdraw_idx),
            Some(after_deposit),
            false,
        );

        balance_array[withdraw_idx] - after_withdrawal
    }

    fn calc_removed_lp_by_withdraw_coins(&self, coins_amounts: [U256;N], max_burn_amount:U256) -> U256{

    }

    fn calc_withdraw_coin_lp_by_removing_lp(&self, coin_amount: U256, token_index: usize) -> (U256,U256,U256){
        let amp = self.amplifier;
        let xp = self.xp();
        let d0 = calculate(xp, amp);

        let total_supply = self.total_supply;
        let d1 = d0 - coin_amount * d0 / total_supply;

        let new_y = get_y(
            xp,
            amp,
            d1,
            token_index,
            None,
            None
            true
        );
        let mut xp_reduced = xp;
        let fee = FEE * N / (4 * (N - 1));
        for j in 0..N {
            let dx_expected = if j == token_index {
                xp[j] * d1 / d0 - new_y
            } else {
                xp[j] - xp[j] * d1 / d0
            };
            xp_reduced[j] -= fee * dx_expected / FEE_DENOMINATOR;
        }
        let mut dy = xp_reduced[token_index] - get_y(
            xp_reduced,
            amp,
            d1,
            token_index,
            None,
            None,
            true
        );
        dy = (dy - 1) / PRECISION_MUL[i];
        let dy_fee = (xp[i] - new_y) / PRECISION_MUL[i];
        (dy, dy_fee, total_supply)
    }

    /// Given a token ID, returns the token position in the array of balances.
    fn token_position(
        &self,
        token_address: Address,
    ) -> usize {
        let mut position = N;
        let mut found = false;

        for i in 0..N {
            if !found{ break; }
            if self.tokens[i] == token_address {
                position = i;
                found = true;
            }
        }

        assert!(found, "The token is not being traded in this pool");

        position
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
