pub mod acc_abstraction_setup;

use acc_abstraction_setup::*;
use account_abstraction::{
    common::common_types::{CallType, GeneralActionData, ScExecutionData, EGLD_TOKEN_ID},
    user_actions::intents::IntentsModule,
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
fn intent_test() {
    let mut setup = AbstractionSetup::new(account_abstraction::contract_obj);

    let first_user_address = setup.first_user.clone();
    let second_user_address = setup.second_user.clone();
    let mock_address = setup.mock_sc_wrapper.address_ref().clone();
    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &multiversx_sc_scenario::rust_biguint!(0),
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
                                endpoint_name: managed_buffer!(DEPOSIT_TOKENS_ENDPOINT_NAME),
                                args,
                                gas_limit: 10_000,
                            }),
                        },
                        0u64,
                        ManagedByteArray::new_from_bytes(EMPTY_SIG),
                    )
                        .into(),
                );

                sc.save_intents(managed_address!(&first_user_address), actions);
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
        .execute_tx(
            &setup.first_user,
            &setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.execute_intent(managed_address!(&first_user_address), 1);
            },
        )
        .assert_ok();

    // check first user tokens after exec
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
