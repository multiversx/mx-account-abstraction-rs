use super::common_types::GeneralActionData;

multiversx_sc::imports!();

pub const SIGNATURE_LEN: usize = 64;
pub type Signature<M> = ManagedByteArray<M, SIGNATURE_LEN>;

static REGISTER_ENDPOINT_NAME: &[u8] = b"registerUser";
static FIELDS_SEPARATOR_CHAR: &[u8] = b"@";

pub type Nonce = u64;
const FIRST_NONCE: Nonce = 0;

pub struct CheckExecutionSignatureArgs<'a, M: ManagedTypeApi> {
    pub own_sc_address: &'a ManagedAddress<M>,
    pub user_address: &'a ManagedAddress<M>,
    pub user_nonce: Nonce,
    pub action: &'a GeneralActionData<M>,
    pub signature: &'a Signature<M>,
}

#[multiversx_sc::module]
pub trait SignatureModule {
    fn check_register_signature(
        &self,
        user_address: &ManagedAddress,
        signature: &Signature<Self::Api>,
    ) {
        let own_sc_address = self.blockchain().get_sc_address();
        let mut signature_data = ManagedBuffer::new_from_bytes(REGISTER_ENDPOINT_NAME);
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append(user_address.as_managed_buffer());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append(own_sc_address.as_managed_buffer());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append_bytes(&FIRST_NONCE.to_be_bytes());

        // TODO: Update to Secp256r1 when it's available
        self.crypto().verify_ed25519(
            user_address.as_managed_buffer(),
            &signature_data,
            signature.as_managed_buffer(),
        );
    }

    fn check_execution_signature(&self, args: CheckExecutionSignatureArgs<Self::Api>) {
        let mut serialized_action = ManagedBuffer::new();
        let encode_result = args.action.top_encode(&mut serialized_action);
        require!(encode_result.is_ok(), "Encoding error");

        let mut signature_data = ManagedBuffer::new();
        signature_data.append(args.user_address.as_managed_buffer());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append(args.own_sc_address.as_managed_buffer());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append_bytes(&args.user_nonce.to_be_bytes());
        signature_data.append_bytes(FIELDS_SEPARATOR_CHAR);
        signature_data.append(&serialized_action);

        // TODO: Update to Secp256r1 when it's available
        self.crypto().verify_ed25519(
            args.user_address.as_managed_buffer(),
            &signature_data,
            args.signature.as_managed_buffer(),
        );
    }
}
