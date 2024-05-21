use farm::base_functions::ClaimRewardsResultType;

use super::common_types::PaymentsVec;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CustomCallbacksModule:
    super::users::UsersModule + super::signature::SignatureModule + utils::UtilsModule
{
    #[callback]
    fn user_action_cb(
        &self,
        original_user: ManagedAddress,
        original_payments: PaymentsVec<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        if call_result.is_ok() {
            return;
        }

        self.refund_user(&original_user, &original_payments);
    }

    #[callback]
    fn claim_farm_rew_cb(
        &self,
        original_user: ManagedAddress,
        original_payments: PaymentsVec<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<ClaimRewardsResultType<Self::Api>>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(multi_result) => {
                let (new_farm_token, rewards) = multi_result.into_tuple();
                let mut all_tokens = ManagedVec::from_single_item(new_farm_token);
                all_tokens.push(rewards);

                self.add_user_funds(&original_user, &all_tokens);
            }
            ManagedAsyncCallResult::Err(_) => self.refund_user(&original_user, &original_payments),
        };
    }

    fn refund_user(&self, user: &ManagedAddress, original_payments: &PaymentsVec<Self::Api>) {
        let user_id = self.user_ids().get_id(user);
        self.user_tokens(user_id).update(|user_tokens| {
            for payment in original_payments {
                user_tokens.add_payment(payment);
            }
        });
    }

    #[inline]
    fn add_user_funds(&self, user: &ManagedAddress, payments: &PaymentsVec<Self::Api>) {
        self.refund_user(user, payments);
    }
}
