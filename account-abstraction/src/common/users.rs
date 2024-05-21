use mergeable::Mergeable;

use super::{
    signature::{Nonce, Signature},
    unique_payments::{PaymentsVec, UniquePayments},
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UsersModule: super::signature::SignatureModule + utils::UtilsModule {
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
        let payments = self.get_non_empty_payments();
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
