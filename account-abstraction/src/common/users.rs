use mergeable::Mergeable;

use super::{
    common_types::{PaymentsVec, UniquePayments, EGLD_TOKEN_ID},
    signature::{Nonce, Signature},
};

multiversx_sc::imports!();

static NOT_ENOUGH_TOKENS_ERR_MSG: &[u8] = b"Not enough tokens";

#[multiversx_sc::module]
pub trait UsersModule: super::signature::SignatureModule {
    #[endpoint(registerUser)]
    fn register_user(&self, user_address: ManagedAddress, signature: Signature<Self::Api>) {
        self.require_not_registered(&user_address);
        self.check_register_signature(&user_address, &signature);

        let _user_id = self.user_ids().insert_new(&user_address);
        // if first nonce ever changes, uncomment this
        // self.user_nonce(user_id).set(FIRST_NONCE);
    }

    #[payable("*")]
    #[endpoint(depositForUser)]
    fn deposit_for_user(&self, user_address: ManagedAddress) {
        let user_id = self.user_ids().get_id_non_zero(&user_address);
        let payments = self.get_esdt_and_egld_payments();
        require!(!payments.is_empty(), "No payments");

        let unique_payments = UniquePayments::new_from_payments(payments);
        let mapper = self.user_tokens(user_id);
        let mut user_tokens = self.get_or_default(&mapper);
        user_tokens.merge_with(unique_payments);
        mapper.set(user_tokens);
    }

    #[view(getUserTokens)]
    fn get_user_tokens(&self, user_address: ManagedAddress) -> PaymentsVec<Self::Api> {
        let user_id = self.user_ids().get_id_non_zero(&user_address);
        let mapper = self.user_tokens(user_id);
        let user_tokens = self.get_or_default(&mapper);

        user_tokens.into_payments()
    }

    #[view(getUserNonce)]
    fn get_user_nonce(&self, user_address: ManagedAddress) -> Nonce {
        let user_id = self.user_ids().get_id_non_zero(&user_address);

        self.user_nonce(user_id).get()
    }

    fn get_or_default(
        &self,
        mapper: &SingleValueMapper<UniquePayments<Self::Api>>,
    ) -> UniquePayments<Self::Api> {
        if !mapper.is_empty() {
            mapper.get()
        } else {
            UniquePayments::new()
        }
    }

    fn get_esdt_and_egld_payments(&self) -> PaymentsVec<Self::Api> {
        let mut payments = self.call_value().all_esdt_transfers().clone_value();
        let egld_value = self.call_value().egld_value().clone_value();
        if egld_value > 0 {
            payments.push(EsdtTokenPayment::new(
                TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID),
                0,
                egld_value,
            ));
        }

        payments
    }

    fn deduct_single_payment(&self, user_id: AddressId, tokens: &EsdtTokenPayment) {
        self.user_tokens(user_id).update(|user_tokens| {
            let deduct_result = user_tokens.deduct_payment(tokens);
            require!(deduct_result.is_ok(), NOT_ENOUGH_TOKENS_ERR_MSG);
        });
    }

    fn deduct_payments(
        &self,
        action_payments: &PaymentsVec<Self::Api>,
        user_tokens: &mut UniquePayments<Self::Api>,
    ) {
        for payment in action_payments {
            let deduct_result = user_tokens.deduct_payment(&payment);
            require!(deduct_result.is_ok(), NOT_ENOUGH_TOKENS_ERR_MSG);
        }
    }

    fn require_not_registered(&self, user_address: &ManagedAddress) {
        require!(
            self.user_ids().get_id(user_address) == NULL_ID,
            "User already registered"
        );
    }

    #[storage_mapper("userIds")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("userTokens")]
    fn user_tokens(&self, user_id: AddressId) -> SingleValueMapper<UniquePayments<Self::Api>>;

    #[storage_mapper("userNonce")]
    fn user_nonce(&self, user_id: AddressId) -> SingleValueMapper<Nonce>;
}
