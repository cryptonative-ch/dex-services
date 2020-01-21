use crate::*;

use ethcontract::web3::api::Web3;
use ethcontract::web3::transports::Http;
use ethcontract::web3::types::{H160, U256};
use ethcontract::Account;

use crate::common::{
    approve, create_accounts_with_funded_tokens, wait_for, FutureWaitExt, MAX_GAS, TOKEN_MINTED,
};

pub fn setup_stablex(
    web3: &Web3<Http>,
    num_tokens: usize,
    num_users: usize,
) -> (BatchExchange, Vec<H160>, Vec<IERC20>) {
    // Get all tokens but OWL in a generic way
    let (accounts, mut tokens) =
        create_accounts_with_funded_tokens(&web3, num_tokens - 1, num_users);
    let mut instance = BatchExchange::deployed(&web3)
        .wait()
        .expect("Cannot get deployed BatchExchange");
    instance.defaults_mut().gas = Some(MAX_GAS.into());
    approve(&tokens, instance.address(), &accounts);

    // Set up OWL manually
    let owl_address = instance
        .token_id_to_address_map(0)
        .call()
        .wait()
        .expect("Cannot get address of OWL Token");
    let owl = TokenOWL::at(web3, owl_address);
    owl.set_minter(accounts[0])
        .send()
        .wait()
        .expect("Cannot set minter");
    for account in &accounts {
        owl.mint_owl(*account, U256::exp10(18) * TOKEN_MINTED)
            .send()
            .wait()
            .expect("Cannot mint OWl");
        owl.approve(instance.address(), U256::exp10(18) * TOKEN_MINTED)
            .from(Account::Local(*account, None))
            .send()
            .wait()
            .expect("Cannot approve OWL for burning");
    }

    // token[0] is already added in constructor
    for token in &tokens {
        instance
            .add_token(token.address())
            .gas(MAX_GAS.into())
            .send()
            .wait()
            .expect("Cannot add token");
    }
    tokens.insert(0, IERC20::at(&web3, owl_address));
    (instance, accounts, tokens)
}

pub fn close_auction(web3: &Web3<Http>, instance: &BatchExchange) {
    let seconds_remaining = instance
        .get_seconds_remaining_in_batch()
        .call()
        .wait()
        .expect("Cannot get seconds remaining in batch");
    wait_for(web3, seconds_remaining.as_u32());
}