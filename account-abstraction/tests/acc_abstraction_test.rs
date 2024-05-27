pub mod acc_abstraction_setup;
use acc_abstraction_setup::*;
use account_abstraction::{
    common::common_types::{CallType, GeneralActionData, EGLD_TOKEN_ID},
    user_actions::execution::ExecutionModule,
};
use multiversx_sc::types::{EsdtTokenPayment, ManagedByteArray, ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::{
    imports::TxTokenTransfer, managed_address, managed_biguint, managed_token_id, rust_biguint,
};

#[test]
fn setup_test() {
    let _ = AbstractionSetup::new(account_abstraction::contract_obj);
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
