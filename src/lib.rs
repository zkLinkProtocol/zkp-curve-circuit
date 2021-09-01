pub mod exchanges;
pub mod invariant;

use primitive_types::{U256,H160};

// The ETH address in the unsigned integer form.
pub type Balance = U256;

//The biggest integer type available to store balances
pub type Address = H160;
pub const ZERO: Balance = U256::from(0);
// The default computation precision.
// Balances are multiplied by it during invariant computation,
// to avoid division of integers of the same order of magnitude.
pub const PRECISION_MUL: U256 = U256::from(1_000_000);
// The maximum known token precision.
pub const MAX_TOKEN_PRECISION: u8 = 18;
// The number of tokens being traded in the pool.
pub const N: usize = 2;

struct StableSwap{
    // The tokens being traded in the pool.
    tokens: [Address; N],
    // The Curve amplifier.
    amplifier: u64,
    // The balances of tokens
    balances: [Balance; N]
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
            balances
        }
    }

    /// Adds liquidity to the contract balances.
    pub fn add_liquidity(&mut self, amount: U256, min_amounts: [U256;N]) -> [U256;N] {

        // panics if the token with address `zksync::msg.token_address` is not traded in this pool
        let deposit_idx = self.token_position(Address::from_address(zksync::msg.token_address));
    }

    /// Removes liquidity to the contract balances.
    pub fn remove_liquidity(&mut self, amount: U256, min_amounts: [U256;N]) -> [U256;N]{

        // panics if the token with address `zksync::msg.token_address` is not traded in this pool
        let deposit_idx = self.token_position(Address::from_address(zksync::msg.token_address));
    }

    ///
    /// Exchanges the tokens, consuming some of the `zksync::msg.token_address` and returning
    /// some of the `withdraw_token_address` to the client.
    ///
    pub fn swap(
        mut self,
        withdraw_address: Address,
        withdraw_token_address: Address,
        min_withdraw: Balance,
    ) {
        assert!(
            zksync::msg.recipient == self.address,
            "Transaction recipient is not the contract",
        );

        let deposit_idx = self.token_position(Address::from_address(zksync::msg.token_address));
        let withdraw_idx = self.token_position(withdraw_token_address);

        let balance_array = self.balances;

        assert_ne!(balance_array[deposit_idx], 0, "Deposit token balance is zero");
        assert_ne!(balance_array[withdraw_idx], 0, "Withdraw token balance is zero");

        let new_x = balance_array[deposit_idx] + zksync::msg.amount;
        let new_y = exchanges::after(
            self.tokens,
            balance_array,
            self.amplifier,
            deposit_idx,
            withdraw_idx,
            new_x,
        );

        let old_y = balance_array[withdraw_idx];
        assert!(
            old_y >= min_withdraw + new_y,
            "Exchange resulted in fewer coins than expected",
        );
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

        let after_withdrawal = balance_array[withdraw_idx] - to_withdraw;

        let after_deposit = exchanges::after(
            self.tokens,
            balance_array,
            self.amplifier,
            withdraw_idx,
            deposit_idx,
            after_withdrawal,
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

        let after_deposit = balance_array[deposit_idx] + to_deposit;

        let after_withdrawal = exchanges::after(
            self.tokens,
            self.balances,
            self.amplifier,
            deposit_idx,
            withdraw_idx,
            after_deposit,
        );

        balance_array[withdraw_idx] - after_withdrawal
    }

    ///
    /// Given a token ID, returns the token position in the array of balances.
    ///
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
