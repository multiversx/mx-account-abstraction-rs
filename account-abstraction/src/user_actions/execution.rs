use crate::{
    signature::CheckExecutionSignatureArgs,
    unique_payments::{PaymentsVec, UniquePayments},
};

use super::common_types::{ActionMultiValue, CallType, GeneralActionData, TxType};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecutionModule:
    crate::users::UsersModule + crate::signature::SignatureModule + utils::UtilsModule
{
    #[endpoint(multiActionForUser)]
    fn multi_action_for_user(
        &self,
        user_address: ManagedAddress,
        actions: MultiValueEncoded<ActionMultiValue<Self::Api>>,
    ) {
        self.require_non_empty_actions(&actions);

        let own_sc_address = self.blockchain().get_sc_address();
        self.multi_action_for_user_common(&user_address, actions, &own_sc_address);
    }

    fn multi_action_for_user_common(
        &self,
        user_address: &ManagedAddress,
        actions: MultiValueEncoded<ActionMultiValue<Self::Api>>,
        own_sc_address: &ManagedAddress,
    ) {
        let user_id = self.user_ids().get_id_non_zero(user_address);
        let nonce_mapper = self.user_nonce(user_id);
        let mut user_nonce = nonce_mapper.get();
        let tokens_mapper = self.user_tokens(user_id);
        let mut user_tokens = tokens_mapper.get();
        for action_multi in actions {
            let (action, nonce, signature) = action_multi.into_tuple();
            require!(nonce == user_nonce, "Invalid user nonce");
            require!(
                &action.dest_address != own_sc_address,
                "Invalid destination"
            );
            require!(!action.is_banned_endpoint_name(), "Invalid endpoint name");

            let args = CheckExecutionSignatureArgs {
                own_sc_address,
                user_address,
                user_nonce,
                action: &action,
                signature: &signature,
            };
            self.check_execution_signature(args);
            self.check_exec_args(&action);
            self.deduct_payments(&action.payments, &mut user_tokens);

            match action.call_type {
                CallType::Transfer => self
                    .tx()
                    .to(action.dest_address)
                    .multi_esdt(action.payments)
                    .transfer(),
                CallType::Sync => {
                    let tx = self.build_tx(action);
                    tx.sync_call();
                }
                CallType::Async => {
                    let original_payments = action.payments.clone();
                    let tx = self.build_tx(action);
                    tx.with_callback(
                        self.callbacks()
                            .user_action_cb(user_address.clone(), original_payments),
                    )
                    .register_promise();
                }
            };

            user_nonce += 1;
        }

        nonce_mapper.set(user_nonce);
        tokens_mapper.set(user_tokens);
    }

    fn check_exec_args(&self, action: &GeneralActionData<Self::Api>) {
        if !self.blockchain().is_smart_contract(&action.dest_address) {
            require!(
                action.opt_execution.is_none(),
                "May not use call data for user transfers"
            );
        }
    }

    fn deduct_payments(
        &self,
        action_payments: &PaymentsVec<Self::Api>,
        user_tokens: &mut UniquePayments<Self::Api>,
    ) {
        require!(!action_payments.is_empty(), "No payments for action");

        for payment in action_payments {
            let deduct_result = user_tokens.deduct_payment(&payment);
            require!(deduct_result.is_ok(), "Not enough tokens");
        }
    }

    fn build_tx(&self, action_data: GeneralActionData<Self::Api>) -> TxType<Self::Api> {
        require!(action_data.opt_execution.is_some(), "Invalid Tx data");

        let sc_exec_data = unsafe { action_data.opt_execution.unwrap_unchecked() };
        self.tx()
            .to(action_data.dest_address)
            .multi_esdt(action_data.payments)
            .raw_call(sc_exec_data.endpoint_name)
            .arguments_raw(sc_exec_data.args.into())
            .gas(sc_exec_data.gas_limit)
    }

    fn require_non_empty_actions(&self, actions: &MultiValueEncoded<ActionMultiValue<Self::Api>>) {
        require!(!actions.is_empty(), "No actions");
    }

    #[callback]
    fn user_action_cb(
        &self,
        original_caller: ManagedAddress,
        original_payments: PaymentsVec<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        if call_result.is_err() {
            self.tx()
                .to(original_caller)
                .multi_esdt(original_payments)
                .transfer();
        }
    }
}
