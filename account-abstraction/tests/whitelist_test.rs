pub mod acc_abstraction_setup;

use acc_abstraction_setup::*;
use account_abstraction::{
    common::common_types::EGLD_TOKEN_ID, user_actions::whitelist_actions::WhitelistActionsModule,
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{EsdtTokenPayment, ManagedAddress, ManagedBuffer, ManagedVec, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    imports::TxTokenTransfer, managed_address, managed_biguint, managed_buffer, managed_token_id,
    rust_biguint, DebugApi,
};

#[test]
fn whitelist_actions_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    let mock_address = setup.mock_sc_wrapper.address_ref().clone();

    // try execute action without whitelist
    setup
        .b_mock
        .execute_tx(
            &second_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));
                sc.take_action(
                    managed_address!(&first_user_address),
                    managed_address!(&mock_address),
                    managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                    args,
                    OptionalValue::Some(EsdtTokenPayment::new(
                        managed_token_id!(EGLD_TOKEN_ID),
                        0,
                        managed_biguint!(100),
                    )),
                );
            },
        )
        .assert_user_error("Unknown address");

    // whitelist user
    setup
        .b_mock
        .execute_tx(
            &first_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut action_types = MultiValueEncoded::new();
                action_types.push(
                    (
                        managed_address!(&mock_address),
                        managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                    )
                        .into(),
                );

                sc.whitelist(managed_address!(&second_user_address), action_types);
            },
        )
        .assert_ok();

    // try execute action - wrong endpoint
    setup
        .b_mock
        .execute_tx(
            &second_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));
                sc.take_action(
                    managed_address!(&first_user_address),
                    managed_address!(&mock_address),
                    managed_buffer!(b"randomEndpointName"),
                    args,
                    OptionalValue::Some(EsdtTokenPayment::new(
                        managed_token_id!(EGLD_TOKEN_ID),
                        0,
                        managed_biguint!(100),
                    )),
                );
            },
        )
        .assert_user_error("Not whitelisted for action");

    // execute action - ok (EGLD)
    setup
        .b_mock
        .execute_tx(
            &second_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));
                sc.take_action(
                    managed_address!(&first_user_address),
                    managed_address!(&mock_address),
                    managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                    args,
                    OptionalValue::Some(EsdtTokenPayment::new(
                        managed_token_id!(EGLD_TOKEN_ID),
                        0,
                        managed_biguint!(100),
                    )),
                );
            },
        )
        .assert_ok();

    // check first user tokens
    let expected_first_user_tokens = [
        TxTokenTransfer {
            token_identifier: EGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_EGLD_BALANCE - 100),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_ESDT_BALANCE),
        },
    ];
    setup.check_user_tokens(&first_user_address, &expected_first_user_tokens);

    // check second user tokens in mock
    let expected_second_user_tokens = [TxTokenTransfer {
        token_identifier: EGLD_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(100),
    }];
    setup.check_user_tokens_mock(&second_user_address, &expected_second_user_tokens);

    // execute action - ok (ESDT)
    setup
        .b_mock
        .execute_tx(
            &second_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));
                sc.take_action(
                    managed_address!(&first_user_address),
                    managed_address!(&mock_address),
                    managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                    args,
                    OptionalValue::Some(EsdtTokenPayment::new(
                        managed_token_id!(TOKEN_ID),
                        0,
                        managed_biguint!(200),
                    )),
                );
            },
        )
        .assert_ok();

    // check first user tokens
    let expected_first_user_tokens = [
        TxTokenTransfer {
            token_identifier: EGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_EGLD_BALANCE - 100),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_ESDT_BALANCE - 200),
        },
    ];
    setup.check_user_tokens(&first_user_address, &expected_first_user_tokens);

    // check second user tokens in mock
    let expected_second_user_tokens = [
        TxTokenTransfer {
            token_identifier: EGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(100),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(200),
        },
    ];
    setup.check_user_tokens_mock(&second_user_address, &expected_second_user_tokens);

    // remove whitelist
    setup
        .b_mock
        .execute_tx(
            &first_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut action_types = MultiValueEncoded::new();
                action_types.push(
                    (
                        managed_address!(&mock_address),
                        managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                    )
                        .into(),
                );

                sc.remove_whitelist(managed_address!(&second_user_address), action_types);
            },
        )
        .assert_ok();

    // try execute action after removed whitelist
    setup
        .b_mock
        .execute_tx(
            &second_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));
                sc.take_action(
                    managed_address!(&first_user_address),
                    managed_address!(&mock_address),
                    managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                    args,
                    OptionalValue::Some(EsdtTokenPayment::new(
                        managed_token_id!(EGLD_TOKEN_ID),
                        0,
                        managed_biguint!(100),
                    )),
                );
            },
        )
        .assert_user_error("Not whitelisted for action");
}
