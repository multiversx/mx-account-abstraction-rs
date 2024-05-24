use crate::user_actions::intents::{IntentId, IntentType};

use super::common_types::PaymentsVec;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CustomCallbacksModule:
    super::users::UsersModule
    + super::signature::SignatureModule
    + crate::user_actions::intent_storage::IntentStorageModule
{
    #[callback]
    fn user_action_cb(
        &self,
        original_user: ManagedAddress,
        original_payments: PaymentsVec<Self::Api>,
        opt_intent_id: Option<IntentId>,
        #[call_result] call_result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(_) => {
                let payments = self.get_esdt_and_egld_payments();
                self.add_user_funds(&original_user, &payments);

                if let Some(intent_id) = opt_intent_id {
                    let user_id = self.user_ids().get_id_non_zero(&original_user);
                    let _ = self.all_user_intents(user_id).swap_remove(&intent_id);
                    self.user_intent(user_id, intent_id).clear();
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.refund_user(&original_user, &original_payments);

                if let Some(intent_id) = opt_intent_id {
                    let user_id = self.user_ids().get_id_non_zero(&original_user);
                    self.user_intent(user_id, intent_id).update(|intent| {
                        intent.intent_type = IntentType::AwaitingExecution;
                    });
                }
            }
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
