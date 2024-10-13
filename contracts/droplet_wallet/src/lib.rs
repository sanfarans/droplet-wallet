#![no_std]

use soroban_sdk::token::TokenClient;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

const OWNER: Symbol = symbol_short!("OWNER");

#[contract]
pub struct DropletWallet;

#[contractimpl]
impl DropletWallet {
    pub fn init(env: Env, owner: Address) {
        if (env.storage().instance().get::<Symbol, Address>(&OWNER)).is_some() {
            panic!("Already initialized");
        }
        env.storage().instance().set(&OWNER, &owner);
        let max_ttl = env.storage().max_ttl();
        env.storage().instance().extend_ttl(max_ttl, max_ttl);
    }

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

    pub fn fund(env: Env, token_address: Address, amount: i128) {
        let owner = Self::auth_owner(&env);
        let token_client = TokenClient::new(&env, &token_address);
        let to = env.current_contract_address();
        token_client.transfer(&owner, &to, &amount);
    }

    pub fn withdraw(env: Env, token_address: Address, amount: i128) {
        let owner = Self::auth_owner(&env);
        let token_client = TokenClient::new(&env, &token_address);
        let from = env.current_contract_address();
        token_client.transfer(&from, &owner, &amount);
    }

    pub fn transfer(env: Env, token_address: Address, to: Address, amount: i128) {
        Self::auth_owner(&env);
        let token_client = TokenClient::new(&env, &token_address);
        let from = env.current_contract_address();
        token_client.transfer(&from, &to, &amount);
    }
}

mod test;
