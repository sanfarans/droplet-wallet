#![no_std]

use core::panic;

use soroban_sdk::token::TokenClient;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

const OWNER: Symbol = symbol_short!("OWNER");
const CHARITY_ADDRESS: Symbol = symbol_short!("CHAR_ADDR");
const CHARITY_FEE: Symbol = symbol_short!("CHAR_PERC");

#[contract]
pub struct DropletWallet;

#[contractimpl]
impl DropletWallet {
    /// Initializes the contract with the given owner address.
    /// This function should be called only once.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment interface for interacting with the blockchain.
    /// * `owner` - The address of the owner who can authorize actions on the contract.
    pub fn init(env: Env, owner: Address) {
        if (env.storage().instance().get::<Symbol, Address>(&OWNER)).is_some() {
            panic!("Already initialized");
        }
        env.storage().instance().set(&OWNER, &owner);
        let max_ttl = env.storage().max_ttl();
        env.storage().instance().extend_ttl(max_ttl, max_ttl);
    }

    /// Authenticates that the caller is the owner.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment interface.
    ///
    /// # Returns
    ///
    /// Returns the owner's address if authenticated.
    fn auth_owner(env: &Env) -> Address {
        if !(env.storage().instance().get::<Symbol, Address>(&OWNER)).is_some() {
            panic!("Not initialized")
        }
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        let max_ttl = env.storage().max_ttl();
        env.storage().instance().extend_ttl(max_ttl, max_ttl);
        owner.require_auth();
        owner
    }

    /// Funds the contract with the specified amount of tokens from the owner.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment interface.
    /// * `token_address` - The address of the token to fund.
    /// * `amount` - The amount of tokens to transfer to the contract.
    pub fn fund(env: Env, token_address: Address, amount: i128) {
        let owner = Self::auth_owner(&env);
        let token_client = TokenClient::new(&env, &token_address);
        let to = env.current_contract_address();
        token_client.transfer(&owner, &to, &amount);
    }

    /// Withdraws the specified amount of tokens from the contract to the owner.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment interface.
    /// * `token_address` - The address of the token to withdraw.
    /// * `amount` - The amount of tokens to transfer to the owner.
    pub fn withdraw(env: Env, token_address: Address, amount: i128) {
        let owner = Self::auth_owner(&env);
        let token_client = TokenClient::new(&env, &token_address);
        let from = env.current_contract_address();
        token_client.transfer(&from, &owner, &amount);
    }

    /// Transfers the specified amount of tokens from the contract to the given address.
    /// If charity address exists and donation fee is positive, a micro-donation is subtracted
    /// from the amount and transferred to charity of choice.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment interface.
    /// * `token_address` - The address of the token to transfer.
    /// * `to` - The address to transfer tokens to.
    /// * `amount` - The amount of tokens to transfer.
    pub fn transfer(env: Env, token_address: Address, to: Address, amount: i128) {
        Self::auth_owner(&env);
        let token_client = TokenClient::new(&env, &token_address);
        let from = env.current_contract_address();

        let fee: Option<i128> = env.storage().instance().get(&CHARITY_FEE);
        let charity_address: Option<Address> = env.storage().instance().get(&CHARITY_ADDRESS);

        if let (Some(fee), Some(charity_address)) = (fee, charity_address) {
            if fee > 0 {
                let fee = amount * fee / 10000;

                if fee > amount {
                    panic!("Amount is less than the calculated fee");
                }

                let amount_after_fee = amount - fee;

                token_client.transfer(&from, &charity_address, &fee);
                token_client.transfer(&from, &to, &amount_after_fee);

                return;
            }
        }

        // If no charity setup or fee is zero, transfer full amount
        token_client.transfer(&from, &to, &amount);
    }

    /// Sets up the charity address and the fee of tokens to donate.
    ///
    /// The fee should be provided in basis points (bips), where 1 bip = 0.01%.
    /// For example, to set a fee of 1%, use 100 bips.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment interface.
    /// * `charity_address` - The address of the charity to donate to.
    /// * `fee` - The percentage (in bips) of tokens to donate.
    pub fn setup_charity(env: Env, charity_address: Address, fee: i128) {
        Self::auth_owner(&env);

        if fee > 10000 {
            panic!("Fee cannot be more than 10000 bips (100%)");
        }
        if fee < 0 {
            panic!("Fee cannot be lower than 0.")
        }

        env.storage()
            .instance()
            .set(&CHARITY_ADDRESS, &charity_address);
        env.storage().instance().set(&CHARITY_FEE, &fee);
    }
}

mod test;
