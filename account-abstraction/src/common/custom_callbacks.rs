use super::common_types::PaymentsVec;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CustomCallbacksModule:
    super::users::UsersModule + super::signature::SignatureModule
{
    #[callback]
    fn user_action_cb(
        &self,
        original_user: ManagedAddress,
        original_payments: PaymentsVec<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(_) => {
                let payments = self.get_esdt_and_egld_payments();
                self.add_user_funds(&original_user, &payments);
            }
            ManagedAsyncCallResult::Err(_) => self.refund_user(&original_user, &original_payments),
        }
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
