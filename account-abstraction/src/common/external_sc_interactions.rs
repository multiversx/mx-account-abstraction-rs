use super::{common_types::GasLimit, custom_callbacks::CallbackProxy as _};

const GAS_TO_SAVE: GasLimit = 100_000;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExternalScInteractionsModule:
    super::custom_callbacks::CustomCallbacksModule
    + super::users::UsersModule
    + super::signature::SignatureModule
{
    fn claim_farm_rewards_promise(
        &self,
        user_address: ManagedAddress,
        farm_token: EsdtTokenPayment,
        farm_address: ManagedAddress,
    ) {
        let gas_limit = self.get_gas_for_promise();
        self.tx()
            .to(farm_address)
            .typed(crate::external_proxies::farm_proxy::FarmProxy)
            .claim_rewards_endpoint(OptionalValue::Some(user_address.clone()))
            .with_esdt_transfer(farm_token.clone())
            .callback(
                self.callbacks()
                    .claim_farm_rew_cb(user_address, ManagedVec::from_single_item(farm_token)),
            )
            .gas(gas_limit)
            .register_promise();
    }

    fn claim_farm_staking_rewards_promise(
        &self,
        user_address: ManagedAddress,
        farm_staking_token: EsdtTokenPayment,
        farm_staking_address: ManagedAddress,
    ) {
        let gas_limit = self.get_gas_for_promise();
        self.tx()
            .to(farm_staking_address)
            .typed(crate::external_proxies::farm_staking_proxy::FarmStakingProxy)
            .claim_rewards(OptionalValue::Some(user_address.clone()))
            .with_esdt_transfer(farm_staking_token.clone())
            .callback(self.callbacks().claim_farm_rew_cb(
                user_address,
                ManagedVec::from_single_item(farm_staking_token),
            ))
            .gas(gas_limit)
            .register_promise();
    }

    fn get_gas_for_promise(&self) -> GasLimit {
        let gas_left = self.blockchain().get_gas_left();
        require!(gas_left > GAS_TO_SAVE, "Not enough gas");

        gas_left - GAS_TO_SAVE
    }
}
