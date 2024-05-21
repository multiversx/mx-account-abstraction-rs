use crate::{
    signature::{Nonce, Signature},
    unique_payments::PaymentsVec,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type GasLimit = u64;
pub type ActionMultiValue<M> = MultiValue3<GeneralActionData<M>, Nonce, Signature<M>>;
pub type TxType<M> = Tx<
    TxScEnv<M>,
    (),
    ManagedAddress<M>,
    PaymentsVec<M>,
    ExplicitGas<GasLimit>,
    FunctionCall<M>,
    (),
>;

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode, ManagedVecItem)]
pub struct ActionStruct<M: ManagedTypeApi> {
    pub action: GeneralActionData<M>,
    pub user_nonce: Nonce,
    pub signature: Signature<M>,
}

const MAX_ENDPOINT_NAME_LEN: usize = 100;
static BANNED_ENDPOINT_NAMES: &[&[u8]] = &[
    b"ESDTLocalMint",
    b"ESDTLocalBurn",
    b"MultiESDTNFTTransfer",
    b"ESDTNFTTransfer",
    b"ESDTNFTCreate",
    b"ESDTNFTAddQuantity",
    b"ESDTNFTAddURI",
    b"ESDTNFTUpdateAttributes",
    b"ESDTNFTBurn",
    b"ESDTTransfer",
    b"ChangeOwnerAddress",
    b"ClaimDeveloperRewards",
    b"SetUserName",
    b"migrateUserName",
    b"DeleteUserName",
    b"upgradeContract",
];

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ScExecutionData<M: ManagedTypeApi> {
    pub endpoint_name: ManagedBuffer<M>,
    pub args: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: GasLimit,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct GeneralActionData<M: ManagedTypeApi> {
    pub call_type: CallType,
    pub dest_address: ManagedAddress<M>,
    pub payments: PaymentsVec<M>,
    pub opt_execution: Option<ScExecutionData<M>>,
}

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy, ManagedVecItem,
)]
pub enum CallType {
    Transfer,
    Sync,
    Async,
}

impl<M: ManagedTypeApi> GeneralActionData<M> {
    pub fn is_banned_endpoint_name(&self) -> bool {
        match &self.opt_execution {
            Some(execution) => {
                let name_len = execution.endpoint_name.len();
                if name_len == 0 {
                    M::error_api_impl().signal_error(b"Empty function name");
                }
                if name_len > MAX_ENDPOINT_NAME_LEN {
                    M::error_api_impl().signal_error(b"Endpoint name too long");
                }

                let mut name_buffer = [0u8; MAX_ENDPOINT_NAME_LEN];
                let copy_result = execution.endpoint_name.load_slice(0, &mut name_buffer);
                if copy_result.is_err() {
                    M::error_api_impl().signal_error(b"Error copying to byte array");
                }

                BANNED_ENDPOINT_NAMES.contains(&&name_buffer[..name_len])
            }
            None => false,
        }
    }
}
