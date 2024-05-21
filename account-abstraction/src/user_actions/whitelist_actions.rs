multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// Note: If adding new actions, always add them to the end to not ruin backwards compatibility
// NEVER delete actions!!!
#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Copy,
)]
pub enum WhitelistActionType {
    ClaimRewardsFarm,
    ClaimRewardsStaking,
    ClaimRewardsDelegation,
    ReDelegateRewards,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct WhitelistAction<M: ManagedTypeApi> {
    pub action_type: WhitelistActionType,
    pub sc_address: ManagedAddress<M>,
}

impl<M: ManagedTypeApi> WhitelistAction<M> {
    #[inline]
    pub fn new(action_type: WhitelistActionType, sc_address: ManagedAddress<M>) -> Self {
        Self {
            action_type,
            sc_address,
        }
    }
}

#[multiversx_sc::module]
pub trait WhitelistActionsModule:
    crate::common::users::UsersModule
    + crate::common::signature::SignatureModule
    + crate::common::custom_callbacks::CustomCallbacksModule
    + crate::common::external_sc_interactions::ExternalScInteractionsModule
    + utils::UtilsModule
{
    /// Pairs of (WhitelistActionType, the SC address for which the whitelist is added)
    #[endpoint]
    fn whitelist(
        &self,
        whitelist_address: ManagedAddress,
        action_types: MultiValueEncoded<MultiValue2<WhitelistActionType, ManagedAddress>>,
    ) {
        self.require_non_empty_action_types(&action_types);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let whitelist_address_id = self.whitelist_ids().get_id_or_insert(&whitelist_address);
        let mut whitelist_mapper = self.user_whitelist(caller_id, whitelist_address_id);
        for multi_value in action_types {
            let (action_type, sc_address) = multi_value.into_tuple();
            let _ = whitelist_mapper.insert(WhitelistAction::new(action_type, sc_address));
        }

        let _ = self
            .all_users_for_whitelist(whitelist_address_id)
            .insert(caller_id);
    }

    /// Pairs of (WhitelistActionType, the SC address for which the whitelist was added)
    #[endpoint(removeWhitelist)]
    fn remove_whitelist(
        &self,
        whitelist_address: ManagedAddress,
        action_types: MultiValueEncoded<MultiValue2<WhitelistActionType, ManagedAddress>>,
    ) {
        self.require_non_empty_action_types(&action_types);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let whitelist_address_id = self.whitelist_ids().get_id_or_insert(&whitelist_address);
        let mut whitelist_mapper = self.user_whitelist(caller_id, whitelist_address_id);
        for multi_value in action_types {
            let (action_type, sc_address) = multi_value.into_tuple();
            let removed =
                whitelist_mapper.swap_remove(&WhitelistAction::new(action_type, sc_address));
            require!(removed, "Address not whitelisted for action");
        }

        if whitelist_mapper.is_empty() {
            let _ = self
                .all_users_for_whitelist(whitelist_address_id)
                .swap_remove(&caller_id);
        }
    }

    #[endpoint(takeAction)]
    fn take_action(
        &self,
        user_address: ManagedAddress,
        action_type: WhitelistActionType,
        sc_address: ManagedAddress,
        opt_user_token: OptionalValue<EsdtTokenPayment>,
    ) {
        let user_id = self.user_ids().get_id_non_zero(&user_address);
        let caller = self.blockchain().get_caller();
        let caller_id = self.whitelist_ids().get_id_non_zero(&caller);
        require!(
            self.user_whitelist(user_id, caller_id)
                .contains(&WhitelistAction::new(action_type, sc_address.clone())),
            "Not whitelisted for action"
        );

        match action_type {
            WhitelistActionType::ClaimRewardsFarm => {
                require!(opt_user_token.is_some(), "Must provide farm token");

                let farm_token = unsafe { opt_user_token.into_option().unwrap_unchecked() };
                self.deduct_single_payment(user_id, &farm_token);
                self.claim_farm_rewards_promise(user_address, farm_token, sc_address);
            }
            WhitelistActionType::ClaimRewardsStaking => {
                require!(opt_user_token.is_some(), "Must provide staking token");

                let farm_staking_token = unsafe { opt_user_token.into_option().unwrap_unchecked() };
                self.deduct_single_payment(user_id, &farm_staking_token);
                self.claim_farm_staking_rewards_promise(
                    user_address,
                    farm_staking_token,
                    sc_address,
                );
            }
            WhitelistActionType::ClaimRewardsDelegation => todo!(),
            WhitelistActionType::ReDelegateRewards => todo!(),
        }
    }

    #[view(getAllWhitelistedUsers)]
    fn get_all_whitelisted_users(
        &self,
        whitelist_address: ManagedAddress,
    ) -> MultiValueEncoded<ManagedAddress> {
        let mut result = MultiValueEncoded::new();
        let whitelist_id = self.whitelist_ids().get_id_non_zero(&whitelist_address);
        let user_id_mapper = self.user_ids();
        for user_id in self.all_users_for_whitelist(whitelist_id).iter() {
            let opt_user_address = user_id_mapper.get_address(user_id);
            require!(opt_user_address.is_some(), "Invalid config");

            let user_address = unsafe { opt_user_address.unwrap_unchecked() };
            result.push(user_address);
        }

        result
    }

    #[view(getWhitelistTypes)]
    fn get_whitelist_types(
        &self,
        whitelist_address: ManagedAddress,
        users: MultiValueEncoded<ManagedAddress>,
    ) -> MultiValueEncoded<ManagedVec<WhitelistAction<Self::Api>>> {
        let mut result = MultiValueEncoded::new();
        let whitelist_id = self.whitelist_ids().get_id_non_zero(&whitelist_address);
        let user_id_mapper = self.user_ids();
        for user in users {
            let user_id = user_id_mapper.get_id_non_zero(&user);

            let mut whitelist_types = ManagedVec::new();
            for whitelist_type in self.user_whitelist(user_id, whitelist_id).iter() {
                whitelist_types.push(whitelist_type);
            }
            result.push(whitelist_types);
        }

        result
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
