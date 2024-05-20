use mergeable::Mergeable;

use crate::unique_payments::{PaymentsVec, UniquePayments};

multiversx_sc::imports!();

pub const SIGNATURE_LEN: usize = 64;
pub type Signature<M> = ManagedByteArray<M, SIGNATURE_LEN>;

static REGISTER_ENDPOINT_NAME: &[u8] = b"registerUser";
static FIELDS_SEPARATOR_CHAR: &[u8] = b"@";

pub type Nonce = u64;
const FIRST_NONCE: Nonce = 0;

#[multiversx_sc::module]
pub trait UsersModule: utils::UtilsModule {
    #[endpoint(registerUser)]
    fn register_user(&self, user_address: ManagedAddress, signature: Signature<Self::Api>) {
        self.require_not_registered(&user_address);
        self.check_signature(&user_address, &signature);

        let _ = self.user_ids().insert_new(&user_address);
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

    fn check_signature(&self, user_address: &ManagedAddress, signature: &Signature<Self::Api>) {
        let own_sc_address = self.blockchain().get_sc_address();
        let mut signature_data = ManagedBuffer::new_from_bytes(REGISTER_ENDPOINT_NAME);
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append(user_address.as_managed_buffer());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append(own_sc_address.as_managed_buffer());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append_bytes(&FIRST_NONCE.to_be_bytes());

        self.crypto().verify_ed25519(
            user_address.as_managed_buffer(),
            &signature_data,
            signature.as_managed_buffer(),
        );
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
