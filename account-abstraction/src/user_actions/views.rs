use super::whitelist_actions::WhitelistAction;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ViewsModule:
    super::whitelist_actions::WhitelistActionsModule
    + crate::common::users::UsersModule
    + crate::common::signature::SignatureModule
    + crate::common::custom_callbacks::CustomCallbacksModule
    + super::execution::ExecutionModule
{
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
}
