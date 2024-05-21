// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            5
// Async Callback:                       1
// Total number of exported functions:   8

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    account_abstraction
    (
        init => init
        upgrade => upgrade
        registerUser => register_user
        depositForUser => deposit_for_user
        getUserTokens => get_user_tokens
        getUserNonce => get_user_nonce
        multiActionForUser => multi_action_for_user
    )
}

multiversx_sc_wasm_adapter::async_callback! { account_abstraction }
