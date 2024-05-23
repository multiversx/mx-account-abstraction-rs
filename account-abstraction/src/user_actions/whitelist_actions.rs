use crate::common::common_types::{
    CallType, GasLimit, GeneralActionData, PaymentsVec, ScExecutionData,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const GAS_TO_SAVE: GasLimit = 100_000;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct WhitelistAction<M: ManagedTypeApi> {
    pub sc_address: ManagedAddress<M>,
    pub endpoint_name: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi> WhitelistAction<M> {
    #[inline]
    pub fn new(sc_address: ManagedAddress<M>, endpoint_name: ManagedBuffer<M>) -> Self {
        Self {
            sc_address,
            endpoint_name,
        }
    }
}

#[multiversx_sc::module]
pub trait WhitelistActionsModule:
    crate::common::users::UsersModule
    + crate::common::signature::SignatureModule
    + crate::common::custom_callbacks::CustomCallbacksModule
    + super::execution::ExecutionModule
    + super::intent_storage::IntentStorageModule
{
    /// Pairs of (SC address, endpoint name)
    #[endpoint]
    fn whitelist(
        &self,
        whitelist_address: ManagedAddress,
        action_types: MultiValueEncoded<MultiValue2<ManagedAddress, ManagedBuffer>>,
    ) {
        self.require_non_empty_action_types(&action_types);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let whitelist_address_id = self.whitelist_ids().get_id_or_insert(&whitelist_address);
        let mut whitelist_mapper = self.user_whitelist(caller_id, whitelist_address_id);
        for multi_value in action_types {
            let (sc_address, endpoint_name) = multi_value.into_tuple();
            let _ = whitelist_mapper.insert(WhitelistAction::new(sc_address, endpoint_name));
        }

        let _ = self
            .all_users_for_whitelist(whitelist_address_id)
            .insert(caller_id);
    }

    /// Pairs of (SC address, endpoint name)
    #[endpoint(removeWhitelist)]
    fn remove_whitelist(
        &self,
        whitelist_address: ManagedAddress,
        action_types: MultiValueEncoded<MultiValue2<ManagedAddress, ManagedBuffer>>,
    ) {
        self.require_non_empty_action_types(&action_types);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let whitelist_address_id = self.whitelist_ids().get_id_non_zero(&whitelist_address);
        let mut whitelist_mapper = self.user_whitelist(caller_id, whitelist_address_id);
        for multi_value in action_types {
            let (sc_address, endpoint_name) = multi_value.into_tuple();
            let removed =
                whitelist_mapper.swap_remove(&WhitelistAction::new(sc_address, endpoint_name));
            require!(removed, "Address not whitelisted for action");
        }

        if whitelist_mapper.is_empty() {
            let _ = self
                .all_users_for_whitelist(whitelist_address_id)
                .swap_remove(&caller_id);
        }
    }

    /// To pass EGLD payment, simply use "EGLD" as token ID, 0 nonce, and the needed amount
    #[endpoint(takeAction)]
    fn take_action(
        &self,
        user_address: ManagedAddress,
        sc_address: ManagedAddress,
        endpoint_name: ManagedBuffer,
        endpoint_args: ManagedVec<ManagedBuffer>,
        opt_user_tokens: OptionalValue<EsdtTokenPayment>,
    ) {
        let user_id = self.user_ids().get_id_non_zero(&user_address);
        let caller = self.blockchain().get_caller();
        let caller_id = self.whitelist_ids().get_id_non_zero(&caller);
        require!(
            self.user_whitelist(user_id, caller_id)
                .contains(&WhitelistAction::new(
                    sc_address.clone(),
                    endpoint_name.clone()
                )),
            "Not whitelisted for action"
        );

        let action_payments = match opt_user_tokens {
            OptionalValue::Some(user_tokens) => {
                self.deduct_single_payment(user_id, &user_tokens);

                PaymentsVec::from_single_item(user_tokens)
            }
            OptionalValue::None => PaymentsVec::new(),
        };

        let gas_limit = self.get_gas_for_promise();
        let action_data = GeneralActionData {
            call_type: CallType::Async,
            dest_address: sc_address,
            payments: action_payments,
            opt_execution: Some(ScExecutionData {
                endpoint_name,
                args: endpoint_args,
                gas_limit,
            }),
        };
        let own_sc_address = self.blockchain().get_sc_address();
        self.multi_action_for_user_common(
            &user_address,
            &ManagedVec::from_single_item(action_data),
            &own_sc_address,
        );
    }

    fn get_gas_for_promise(&self) -> GasLimit {
        let gas_left = self.blockchain().get_gas_left();
        require!(gas_left > GAS_TO_SAVE, "Not enough gas");

        gas_left - GAS_TO_SAVE
    }

    fn require_non_empty_action_types<T>(&self, action_types: &MultiValueEncoded<T>) {
        require!(!action_types.is_empty(), "No whitelist actions");
    }

    #[storage_mapper("whitelistIds")]
    fn whitelist_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(
        &self,
        user_id: AddressId,
        whitelisted_address_id: AddressId,
    ) -> UnorderedSetMapper<WhitelistAction<Self::Api>>;

    #[storage_mapper("allUsersForWhitelist")]
    fn all_users_for_whitelist(
        &self,
        whitelisted_address_id: AddressId,
    ) -> UnorderedSetMapper<AddressId>;
}
