pub mod acc_abstraction_setup;

use acc_abstraction_setup::*;
use account_abstraction::{
    common::{
        common_types::{CallType, GeneralActionData, ScExecutionData, EGLD_TOKEN_ID},
        signature::Signature,
        users::UsersModule,
    },
    user_actions::execution::ExecutionModule,
};
use multiversx_sc::types::{
    EsdtTokenPayment, ManagedAddress, ManagedBuffer, ManagedByteArray, ManagedVec,
    MultiValueEncoded,
};
use multiversx_sc_scenario::{
    imports::TxTokenTransfer, managed_address, managed_biguint, managed_buffer, managed_token_id,
    rust_biguint, DebugApi,
};

#[test]
fn setup_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);
    let first_user_address = setup.first_user.clone();

    // try register first user again
    setup
        .b_mock
        .execute_tx(
            &first_user_address,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.register_user(
                    managed_address!(&first_user_address),
                    Signature::new_from_bytes(EMPTY_SIG),
                );
            },
        )
        .assert_user_error("User already registered");
}

#[test]
fn execute_action_transfer_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut actions = MultiValueEncoded::new();
                actions.push(
                    (
                        GeneralActionData {
                            call_type: CallType::Transfer,
                            dest_address: managed_address!(&second_user_address),
                            payments: ManagedVec::from_single_item(EsdtTokenPayment::new(
                                managed_token_id!(EGLD_TOKEN_ID),
                                0,
                                managed_biguint!(100),
                            )),
                            opt_execution: None,
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.multi_action_for_user(managed_address!(&first_user_address), actions);
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

    setup
        .b_mock
        .check_egld_balance(&second_user_address, &rust_biguint!(100));
}

#[test]
fn execute_action_sc_call_async_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    let mock_address = setup.mock_sc_wrapper.address_ref().clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut actions = MultiValueEncoded::new();
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));

                actions.push(
                    (
                        GeneralActionData {
                            call_type: CallType::Async,
                            dest_address: managed_address!(&mock_address),
                            payments: ManagedVec::from_single_item(EsdtTokenPayment::new(
                                managed_token_id!(EGLD_TOKEN_ID),
                                0,
                                managed_biguint!(100),
                            )),
                            opt_execution: Some(ScExecutionData {
                                endpoint_name: managed_buffer!(b"depositForUser"),
                                args,
                                gas_limit: 10_000,
                            }),
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.multi_action_for_user(managed_address!(&first_user_address), actions);
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
}

#[test]
fn execute_action_sc_call_sync_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    let mock_address = setup.mock_sc_wrapper.address_ref().clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut actions = MultiValueEncoded::new();
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));

                actions.push(
                    (
                        GeneralActionData {
                            call_type: CallType::Sync,
                            dest_address: managed_address!(&mock_address),
                            payments: ManagedVec::from_single_item(EsdtTokenPayment::new(
                                managed_token_id!(EGLD_TOKEN_ID),
                                0,
                                managed_biguint!(100),
                            )),
                            opt_execution: Some(ScExecutionData {
                                endpoint_name: managed_buffer!(b"depositForUser"),
                                args,
                                gas_limit: 10_000,
                            }),
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.multi_action_for_user(managed_address!(&first_user_address), actions);
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
}

#[test]
fn execute_action_transfer_esdt_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut actions = MultiValueEncoded::new();
                actions.push(
                    (
                        GeneralActionData {
                            call_type: CallType::Transfer,
                            dest_address: managed_address!(&second_user_address),
                            payments: ManagedVec::from_single_item(EsdtTokenPayment::new(
                                managed_token_id!(TOKEN_ID),
                                0,
                                managed_biguint!(100),
                            )),
                            opt_execution: None,
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.multi_action_for_user(managed_address!(&first_user_address), actions);
            },
        )
        .assert_ok();

    // check first user tokens
    let expected_first_user_tokens = [
        TxTokenTransfer {
            token_identifier: EGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_EGLD_BALANCE),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_ESDT_BALANCE - 100),
        },
    ];
    setup.check_user_tokens(&first_user_address, &expected_first_user_tokens);

    setup
        .b_mock
        .check_esdt_balance(&second_user_address, TOKEN_ID, &rust_biguint!(100));
}

#[test]
fn execute_action_sc_call_async_esdt_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    let mock_address = setup.mock_sc_wrapper.address_ref().clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut actions = MultiValueEncoded::new();
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));

                actions.push(
                    (
                        GeneralActionData {
                            call_type: CallType::Async,
                            dest_address: managed_address!(&mock_address),
                            payments: ManagedVec::from_single_item(EsdtTokenPayment::new(
                                managed_token_id!(TOKEN_ID),
                                0,
                                managed_biguint!(100),
                            )),
                            opt_execution: Some(ScExecutionData {
                                endpoint_name: managed_buffer!(b"depositForUser"),
                                args,
                                gas_limit: 10_000,
                            }),
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.multi_action_for_user(managed_address!(&first_user_address), actions);
            },
        )
        .assert_ok();

    // check first user tokens
    let expected_first_user_tokens = [
        TxTokenTransfer {
            token_identifier: EGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_EGLD_BALANCE),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_ESDT_BALANCE - 100),
        },
    ];
    setup.check_user_tokens(&first_user_address, &expected_first_user_tokens);

    // check second user tokens in mock
    let expected_second_user_tokens = [TxTokenTransfer {
        token_identifier: TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(100),
    }];
    setup.check_user_tokens_mock(&second_user_address, &expected_second_user_tokens);
}

#[test]
fn execute_action_sc_call_sync_esdt_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    let mock_address = setup.mock_sc_wrapper.address_ref().clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut actions = MultiValueEncoded::new();
                let mut args = ManagedVec::new();
                args.push(ManagedBuffer::new_from_bytes(
                    ManagedAddress::<DebugApi>::from_address(&second_user_address)
                        .to_byte_array()
                        .as_slice(),
                ));

                actions.push(
                    (
                        GeneralActionData {
                            call_type: CallType::Sync,
                            dest_address: managed_address!(&mock_address),
                            payments: ManagedVec::from_single_item(EsdtTokenPayment::new(
                                managed_token_id!(TOKEN_ID),
                                0,
                                managed_biguint!(100),
                            )),
                            opt_execution: Some(ScExecutionData {
                                endpoint_name: managed_buffer!(b"depositForUser"),
                                args,
                                gas_limit: 10_000,
                            }),
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.multi_action_for_user(managed_address!(&first_user_address), actions);
            },
        )
        .assert_ok();

    // check first user tokens
    let expected_first_user_tokens = [
        TxTokenTransfer {
            token_identifier: EGLD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_EGLD_BALANCE),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(FIRST_USER_ESDT_BALANCE - 100),
        },
    ];
    setup.check_user_tokens(&first_user_address, &expected_first_user_tokens);

    // check second user tokens in mock
    let expected_second_user_tokens = [TxTokenTransfer {
        token_identifier: TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(100),
    }];
    setup.check_user_tokens_mock(&second_user_address, &expected_second_user_tokens);
}
