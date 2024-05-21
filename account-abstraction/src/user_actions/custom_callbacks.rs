use crate::unique_payments::PaymentsVec;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CustomCallbacksModule:
    crate::users::UsersModule + crate::signature::SignatureModule + utils::UtilsModule
{
    #[callback]
    fn user_action_cb(
        &self,
        original_caller: ManagedAddress,
        original_payments: PaymentsVec<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        if call_result.is_ok() {
            return;
        }

        let user_id = self.user_ids().get_id(&original_caller);
        self.user_tokens(user_id).update(|user_tokens| {
            for payment in &original_payments {
                user_tokens.add_payment(payment);
            }
        });
    }
}
