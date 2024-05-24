use account_abstraction::common::{
    common_types::{PaymentsVec, EGLD_TOKEN_ID},
    signature::Signature,
};
use multiversx_sc::types::{TestAddress, TestSCAddress};
use multiversx_sc_scenario::imports::*;

pub static OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub static FIRST_USER_ADDRESS: TestAddress = TestAddress::new("firstUser");
pub static SECOND_USER_ADDRESS: TestAddress = TestAddress::new("secondUser");
pub static SC_ADDRESS: TestSCAddress = TestSCAddress::new("accAbstSc");
pub static CODE_PATH: MxscPath = MxscPath::new("output/account-abstraction.mxsc.json");

pub static TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("MYTOK-123456");
pub const FIRST_USER_EGLD_BALANCE: u64 = 500;
pub const FIRST_USER_ESDT_BALANCE: u64 = 1_000;
pub const SECOND_USER_ESDT_BALANCE: u64 = 2_000;
pub static EMPTY_SIG: &[u8; 64] = &[0u8; 64];

pub struct AccAbstractionSetup {
    pub b_mock: ScenarioWorld,
}

impl AccAbstractionSetup {
    pub fn new() -> Self {
        let mut b_mock = ScenarioWorld::new();
        b_mock.register_contract(CODE_PATH, account_abstraction::ContractBuilder);
        b_mock.account(OWNER_ADDRESS);

        b_mock
            .account(FIRST_USER_ADDRESS)
            .balance(FIRST_USER_EGLD_BALANCE)
            .esdt_balance(TOKEN_ID, FIRST_USER_ESDT_BALANCE);
        b_mock
            .account(SECOND_USER_ADDRESS)
            .esdt_balance(TOKEN_ID, SECOND_USER_ESDT_BALANCE);

        b_mock
            .tx()
            .from(OWNER_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .init()
            .code(CODE_PATH)
            .new_address(SC_ADDRESS)
            .run();

        // register first user
        b_mock
            .tx()
            .from(FIRST_USER_ADDRESS)
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .register_user(FIRST_USER_ADDRESS, Signature::new_from_bytes(EMPTY_SIG))
            .run();

        // try register first user again
        b_mock
            .tx()
            .from(FIRST_USER_ADDRESS)
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .register_user(FIRST_USER_ADDRESS, Signature::new_from_bytes(EMPTY_SIG))
            .with_result(ExpectMessage("User already registered"))
            .run();

        // register second user
        b_mock
            .tx()
            .from(SECOND_USER_ADDRESS)
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .register_user(SECOND_USER_ADDRESS, Signature::new_from_bytes(EMPTY_SIG))
            .run();

        // first user deposit EGLD
        b_mock
            .tx()
            .from(FIRST_USER_ADDRESS)
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .deposit_for_user(FIRST_USER_ADDRESS)
            .egld(FIRST_USER_EGLD_BALANCE)
            .run();

        // first user deposit ESDT
        b_mock
            .tx()
            .from(FIRST_USER_ADDRESS)
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .deposit_for_user(FIRST_USER_ADDRESS)
            .single_esdt(&TOKEN_ID.into(), 0u64, &FIRST_USER_ESDT_BALANCE.into())
            .run();

        // second user deposit ESDT
        b_mock
            .tx()
            .from(SECOND_USER_ADDRESS)
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .deposit_for_user(SECOND_USER_ADDRESS)
            .single_esdt(&TOKEN_ID.into(), 0u64, &SECOND_USER_ESDT_BALANCE.into())
            .run();

        // check first user tokens
        let mut expected_first_user_tokens = PaymentsVec::new();
        expected_first_user_tokens.push(EsdtTokenPayment::new(
            TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID),
            0u64,
            FIRST_USER_EGLD_BALANCE.into(),
        ));
        expected_first_user_tokens.push(EsdtTokenPayment::new(
            TOKEN_ID.into(),
            0u64,
            FIRST_USER_ESDT_BALANCE.into(),
        ));
        b_mock
            .query()
            .to(SC_ADDRESS)
            .typed(account_abstraction::own_proxy::AccountAbstractionProxy)
            .get_user_tokens(FIRST_USER_ADDRESS)
            .returns(ExpectValue(expected_first_user_tokens))
            .run();

        Self { b_mock }
    }
}
