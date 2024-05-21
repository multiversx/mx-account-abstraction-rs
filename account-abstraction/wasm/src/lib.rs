// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           11
// Async Callback:                       1
// Total number of exported functions:  14

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
        multiActionForMultiUsers => multi_action_for_multi_users
        whitelist => whitelist
        removeWhitelist => remove_whitelist
        takeAction => take_action
        getAllWhitelistedUsers => get_all_whitelisted_users
        getWhitelistTypes => get_whitelist_types
    )
}

multiversx_sc_wasm_adapter::async_callback! { account_abstraction }
