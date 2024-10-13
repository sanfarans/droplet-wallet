#[cfg(test)]
extern crate std;

use crate::{DropletWallet, DropletWalletClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};
use token::{Client, StellarAssetClient};

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (Client<'a>, StellarAssetClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}

#[test]
fn test_fund_and_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let smart_wallet_id = env.register_contract(None, DropletWallet);
    let smart_wallet_client = DropletWalletClient::new(&env, &smart_wallet_id);

    let alice = Address::generate(&env);
    smart_wallet_client.init(&alice);

    let token_admin = Address::generate(&env);
    let (token_a, token_a_admin) = create_token_contract(&env, &token_admin);
    token_a_admin.mint(&alice, &1000);

    smart_wallet_client.fund(&token_a.address, &1000);
    assert_eq!(token_a.balance(&smart_wallet_id), 1000);
    assert_eq!(token_a.balance(&alice), 0);

    smart_wallet_client.withdraw(&token_a.address, &1000);
    assert_eq!(token_a.balance(&smart_wallet_id), 0);
    assert_eq!(token_a.balance(&alice), 1000);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let smart_wallet_id = env.register_contract(None, DropletWallet);
    let smart_wallet_client = DropletWalletClient::new(&env, &smart_wallet_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let (token_a, token_a_admin) = create_token_contract(&env, &token_admin);
    token_a_admin.mint(&alice, &1000);

    assert_eq!(token_a.balance(&alice), 1000);

    smart_wallet_client.init(&alice);
    let initial_wallet_balance = 500;
    smart_wallet_client.fund(&token_a.address, &initial_wallet_balance);

    assert_eq!(token_a.balance(&smart_wallet_id), 500);
    assert_eq!(token_a.balance(&alice), 500);
    assert_eq!(token_a.balance(&bob), 0);

    let transfer_amount = 200;
    smart_wallet_client.transfer(&token_a.address, &bob, &transfer_amount);

    let to_balance = token_a.balance(&bob);
    assert_eq!(to_balance, transfer_amount);

    let wallet_balance = token_a.balance(&smart_wallet_client.address);
    assert_eq!(wallet_balance, initial_wallet_balance - transfer_amount);
}
