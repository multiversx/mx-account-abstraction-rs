use super::signature::{Nonce, Signature};
use mergeable::Mergeable;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
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

// Stolen from: https://github.com/multiversx/sc-gravity-restaking-rs

#[derive(
    TypeAbi,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    Clone,
    PartialEq,
    Debug,
    ManagedVecItem,
)]
pub struct UniquePayments<M: ManagedTypeApi> {
    payments: PaymentsVec<M>,
}

impl<M: ManagedTypeApi> Default for UniquePayments<M> {
    #[inline]
    fn default() -> Self {
        Self {
            payments: PaymentsVec::new(),
        }
    }
}

impl<M: ManagedTypeApi> UniquePayments<M> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_from_unique_payments(payments: PaymentsVec<M>) -> Self {
        UniquePayments { payments }
    }

    pub fn new_from_payments(payments: PaymentsVec<M>) -> Self {
        let mut merged_payments = Self::new();
        for p in &payments {
            merged_payments.add_payment(p);
        }

        merged_payments
    }

    pub fn add_payment(&mut self, new_payment: EsdtTokenPayment<M>) {
        if new_payment.amount == 0 {
            return;
        }

        let len = self.payments.len();
        for i in 0..len {
            let mut current_payment = self.payments.get(i);
            if !current_payment.can_merge_with(&new_payment) {
                continue;
            }

            current_payment.amount += new_payment.amount;
            let _ = self.payments.set(i, &current_payment);

            return;
        }

        self.payments.push(new_payment);
    }

    #[allow(clippy::result_unit_err)]
    pub fn deduct_payment(&mut self, payment: &EsdtTokenPayment<M>) -> Result<(), ()> {
        if payment.amount == 0 {
            return Result::Ok(());
        }

        let len = self.payments.len();
        for i in 0..len {
            let mut current_payment = self.payments.get(i);
            if !current_payment.can_merge_with(payment) {
                continue;
            }

            if current_payment.amount < payment.amount {
                return Result::Err(());
            }

            current_payment.amount -= &payment.amount;
            if current_payment.amount > 0 {
                let _ = self.payments.set(i, &current_payment);
            } else {
                self.payments.remove(i);
            }

            return Result::Ok(());
        }

        Result::Err(())
    }

    #[inline]
    pub fn into_payments(self) -> PaymentsVec<M> {
        self.payments
    }
}

impl<M: ManagedTypeApi> Mergeable<M> for UniquePayments<M> {
    #[inline]
    fn can_merge_with(&self, _other: &Self) -> bool {
        true
    }

    fn merge_with(&mut self, mut other: Self) {
        self.error_if_not_mergeable(&other);

        if self.payments.is_empty() {
            self.payments = other.payments;
            return;
        }
        if other.payments.is_empty() {
            return;
        }

        let first_len = self.payments.len();
        let mut second_len = other.payments.len();
        for i in 0..first_len {
            let mut current_payment = self.payments.get(i);
            for j in 0..second_len {
                let other_payment = other.payments.get(j);
                if !current_payment.can_merge_with(&other_payment) {
                    continue;
                }

                current_payment.amount += other_payment.amount;
                let _ = self.payments.set(i, &current_payment);

                other.payments.remove(j);
                second_len -= 1;

                break;
            }
        }

        self.payments.append_vec(other.payments);
    }
}
