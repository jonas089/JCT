#![no_std]
#![no_main]

mod constants;
mod utils;
extern crate alloc;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::convert::TryInto;

use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, ApiError, CLType, CLValue,
    ContractHash, ContractPackageHash, ContractVersion, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, KeyTag, Parameter, RuntimeArgs, Tagged, URef,
};

use casper_contract::{
    contract_api::{
        runtime,
        storage::{self, dictionary_get},
    },
    unwrap_or_revert::UnwrapOrRevert,
};

use constants::*;
use utils::*;

// Jonas ERC20 token on Casper

#[no_mangle]
pub extern "C" fn Balance(
    caller_account_key: AccountHash,
    caller_account_key_as_string: &str,
) -> u64 {
    let balances_uref = get_uref("token_balances");
    let _balance = storage::dictionary_get::<u64>(balances_uref, &caller_account_key_as_string);

    let mut __balance: u64 = 0;
    match _balance {
        Ok(maybe_balance) => {
            match maybe_balance {
                Some(balance) => {
                    // Update __balance in outer scope
                    __balance = balance;
                }
                // This should never happen.
                None => {
                    // account not found, not received tokens => balance is 0.
                    __balance = 0;
                }
            }
        }
        Err(_) => {
            // This should never happen, could happen if initialization failed.
            runtime::revert(ApiError::Unhandled)
        }
    }
    __balance
}

#[no_mangle]
pub extern "C" fn mint() {
    let circulating_supply_uref: URef = get_uref("circulating_supply");
    let circulating_supply: u64 = storage::read_or_revert(circulating_supply_uref);
    let max_total_supply_uref: URef = get_uref("max_total_supply");
    let max_total_supply: u64 = storage::read_or_revert(max_total_supply_uref);

    let mint_amount: u64 = 100;
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();
    let balances_uref = get_uref("token_balances");

    // Is the max_supply exceeded by this mint ? - if so, revert.
    if circulating_supply + mint_amount > max_total_supply {
        runtime::revert(ApiError::PermissionDenied)
    }

    // Add mint_amount to account balance.
    let balance_before_mint: u64 = Balance(caller_account_key, &caller_account_key_as_string);
    let balance_after_mint: u64 = balance_before_mint + mint_amount;
    storage::dictionary_put(
        balances_uref,
        &caller_account_key_as_string,
        balance_after_mint,
    );
    let updated_circulating_supply: u64 = circulating_supply + mint_amount;
    storage::write(circulating_supply_uref, updated_circulating_supply);
}

#[no_mangle]
pub extern "C" fn balanceOf() {
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();

    let __balance: u64 = Balance(caller_account_key, &caller_account_key_as_string);
    let balance_uref = storage::new_uref(__balance);
    runtime::put_key("balance", balance_uref.into())
}

#[no_mangle]
pub extern "C" fn call() {
    // Constants for testing:
    let amount_mints: u64 = 11; // given a max supply of 1000 and a mint_amount of 100, 10 is the maximum possible.
                                // any value above 10 will cause a revert with a PermissionDenied error.

    // initialize token
    storage::new_dictionary("token_balances");
    // currently not used for testing.

    runtime::put_key(
        "circulating_supply",
        storage::new_uref("circulating_supply").into(),
    );
    let circulating_supply_uref: URef = get_uref("circulating_supply");
    let circulating_supply: u64 = 0;
    storage::write(circulating_supply_uref, circulating_supply);

    runtime::put_key(
        "max_total_supply",
        storage::new_uref("max_total_supply").into(),
    );
    let max_total_supply_uref: URef = get_uref("max_total_supply");
    let max_total_supply: u64 = 1000;
    storage::write(max_total_supply_uref, max_total_supply);
    // test the raised error that'll be thrown when max_supply is exceeded.
    /*let circ_supply: u64 = storage::read_or_revert(circulating_supply_uref);
    let compare_to: u64 = 1000;
    if circ_supply != compare_to {
        runtime::revert(ApiError::PermissionDenied)
    }*/

    // call dummy mint function
    for i in (0..amount_mints) {
        mint();
    }
    balanceOf()
}
