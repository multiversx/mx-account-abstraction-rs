use account_abstraction::{
    common::{
        common_types::{PaymentsVec, EGLD_TOKEN_ID},
        signature::Signature,
        users::UsersModule,
    },
    AccountAbstraction,
};
use multiversx_sc::types::{Address, EsdtTokenPayment};
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper, TxTokenTransfer},
    managed_address, managed_token_id, rust_biguint, DebugApi,
};

pub static TOKEN_ID: &[u8] = b"MYTOK-123456";
pub const FIRST_USER_EGLD_BALANCE: u64 = 500;
pub const FIRST_USER_ESDT_BALANCE: u64 = 1_000;
pub const SECOND_USER_ESDT_BALANCE: u64 = 2_000;
pub static EMPTY_SIG: &[u8; 64] = &[0u8; 64];

pub static DEPOSIT_TOKENS_ENDPOINT_NAME: &[u8] = b"depositForUser";

pub struct AbstractionSetup<AbstractionBuilder>
where
    AbstractionBuilder: 'static + Copy + Fn() -> account_abstraction::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner: Address,
    pub first_user: Address,
    pub second_user: Address,
    pub sc_wrapper:
        ContractObjWrapper<account_abstraction::ContractObj<DebugApi>, AbstractionBuilder>,
    pub mock_sc_wrapper:
        ContractObjWrapper<account_abstraction::ContractObj<DebugApi>, AbstractionBuilder>,
}

impl<AbstractionBuilder> AbstractionSetup<AbstractionBuilder>
where
    AbstractionBuilder: 'static + Copy + Fn() -> account_abstraction::ContractObj<DebugApi>,
{
    pub fn new(abstraction_builder: AbstractionBuilder) -> Self {
        let rust_zero = rust_biguint!(0);

        let mut b_mock = BlockchainStateWrapper::new();
        let owner = b_mock.create_user_account(&rust_zero);
        let first_user = b_mock.create_user_account(&rust_biguint!(FIRST_USER_EGLD_BALANCE));
        let second_user = b_mock.create_user_account(&rust_zero);
        b_mock.set_esdt_balance(
            &first_user,
            TOKEN_ID,
            &rust_biguint!(FIRST_USER_ESDT_BALANCE),
        );
        b_mock.set_esdt_balance(
            &second_user,
            TOKEN_ID,
            &rust_biguint!(SECOND_USER_ESDT_BALANCE),
        );

        let sc_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner), abstraction_builder, "abstraction");

        let mock_sc_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner), abstraction_builder, "mock sc");

        b_mock
            .execute_tx(&owner, &sc_wrapper, &rust_zero, |sc| {
                sc.init();
            })
            .assert_ok();

        b_mock
            .execute_tx(&owner, &mock_sc_wrapper, &rust_zero, |sc| {
                sc.init();
            })
            .assert_ok();

        // register first user
        b_mock
            .execute_tx(&first_user, &sc_wrapper, &rust_zero, |sc| {
                sc.register_user(
                    managed_address!(&first_user),
                    Signature::new_from_bytes(EMPTY_SIG),
                );
            })
            .assert_ok();

        // register second user
        b_mock
            .execute_tx(&second_user, &sc_wrapper, &rust_zero, |sc| {
                sc.register_user(
                    managed_address!(&second_user),
                    Signature::new_from_bytes(EMPTY_SIG),
                );
            })
            .assert_ok();

        // register second user in mock
        b_mock
            .execute_tx(&second_user, &mock_sc_wrapper, &rust_zero, |sc| {
                sc.register_user(
                    managed_address!(&second_user),
                    Signature::new_from_bytes(EMPTY_SIG),
                );
            })
            .assert_ok();

        // first user deposit EGLD
        b_mock
            .execute_tx(
                &first_user,
                &sc_wrapper,
                &rust_biguint!(FIRST_USER_EGLD_BALANCE),
                |sc| {
                    sc.deposit_for_user(managed_address!(&first_user));
                },
            )
            .assert_ok();

        // first user deposit ESDT
        b_mock
            .execute_esdt_transfer(
                &first_user,
                &sc_wrapper,
                TOKEN_ID,
                0,
                &rust_biguint!(FIRST_USER_ESDT_BALANCE),
                |sc| {
                    sc.deposit_for_user(managed_address!(&first_user));
                },
            )
            .assert_ok();

        // second user deposit ESDT
        b_mock
            .execute_esdt_transfer(
                &second_user,
                &sc_wrapper,
                TOKEN_ID,
                0,
                &rust_biguint!(SECOND_USER_ESDT_BALANCE),
                |sc| {
                    sc.deposit_for_user(managed_address!(&second_user));
                },
            )
            .assert_ok();

        let mut own_inst = Self {
            b_mock,
            owner,
            first_user,
            second_user,
            sc_wrapper,
            mock_sc_wrapper,
        };

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
                value: rust_biguint!(FIRST_USER_ESDT_BALANCE),
            },
        ];
        own_inst.check_user_tokens(&own_inst.first_user.clone(), &expected_first_user_tokens);

        own_inst
    }

    pub fn check_user_tokens(&mut self, user: &Address, expected_tokens: &[TxTokenTransfer]) {
        Self::check_tokens_common(&mut self.b_mock, &self.sc_wrapper, user, expected_tokens);
    }

    pub fn check_user_tokens_mock(&mut self, user: &Address, expected_tokens: &[TxTokenTransfer]) {
        Self::check_tokens_common(
            &mut self.b_mock,
            &self.mock_sc_wrapper,
            user,
            expected_tokens,
        );
    }

    fn check_tokens_common(
        b_mock: &mut BlockchainStateWrapper,
        sc_wrapper: &ContractObjWrapper<
            account_abstraction::ContractObj<DebugApi>,
            AbstractionBuilder,
        >,
        user: &Address,
        expected_tokens: &[TxTokenTransfer],
    ) {
        b_mock
            .execute_query(sc_wrapper, |sc| {
                let mut expected_tokens_managed = PaymentsVec::new();
                for expected_token in expected_tokens {
                    expected_tokens_managed.push(EsdtTokenPayment::new(
                        managed_token_id!(expected_token.token_identifier.clone()),
                        expected_token.nonce,
                        multiversx_sc::types::BigUint::from_bytes_be(
                            expected_token.value.to_bytes_be().as_slice(),
                        ),
                    ));
                }

                let actual_user_tokens = sc.get_user_tokens(managed_address!(&user));
                assert_eq!(expected_tokens_managed, actual_user_tokens);
            })
            .assert_ok();
    }
}
