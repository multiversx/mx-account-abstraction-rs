use crate::common::{custom_callbacks::CallbackProxy as _, signature::CheckExecutionSignatureArgs};

use crate::common::common_types::{
    ActionMultiValue, ActionStruct, CallType, GasLimit, GeneralActionData, TxType,
};

const DEFAULT_EXTRA_CALLBACK_GAS: GasLimit = 10_000_000;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecutionModule:
    crate::common::users::UsersModule
    + crate::common::signature::SignatureModule
    + crate::common::custom_callbacks::CustomCallbacksModule
    + utils::UtilsModule
{
    #[endpoint(multiActionForUser)]
    fn multi_action_for_user(
        &self,
        user_address: ManagedAddress,
        actions: MultiValueEncoded<ActionMultiValue<Self::Api>>,
    ) {
        self.require_non_empty_actions(&actions);

        let own_sc_address = self.blockchain().get_sc_address();
        let mut actions_vec = ManagedVec::new();
        for action_multi in actions {
            let (action, user_nonce, signature) = action_multi.into_tuple();
            let action_struct = ActionStruct {
                action,
                user_nonce,
                signature,
            };
            actions_vec.push(action_struct);
        }
        self.multi_action_for_user_common(&user_address, &actions_vec, &own_sc_address);
    }

    /// Pairs of (user_address, actions_vec)
    #[endpoint(multiActionForMultiUsers)]
    fn multi_action_for_multi_users(
        &self,
        args: MultiValueEncoded<MultiValue2<ManagedAddress, ManagedVec<ActionStruct<Self::Api>>>>,
    ) {
        self.require_non_empty_actions(&args);

        let own_sc_address = self.blockchain().get_sc_address();
        for pair in args {
            let (user_address, actions_vec) = pair.into_tuple();
            self.multi_action_for_user_common(&user_address, &actions_vec, &own_sc_address);
        }
    }

    fn multi_action_for_user_common(
        &self,
        user_address: &ManagedAddress,
        actions: &ManagedVec<ActionStruct<Self::Api>>,
        own_sc_address: &ManagedAddress,
    ) {
        let user_id = self.user_ids().get_id_non_zero(user_address);
        let nonce_mapper = self.user_nonce(user_id);
        let mut user_nonce = nonce_mapper.get();
        let tokens_mapper = self.user_tokens(user_id);
        let mut user_tokens = tokens_mapper.get();
        for action_struct in actions {
            let (action, nonce, signature) = (
                action_struct.action,
                action_struct.user_nonce,
                action_struct.signature,
            );
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
                    .with_extra_gas_for_callback(DEFAULT_EXTRA_CALLBACK_GAS)
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

    fn require_non_empty_actions<T>(&self, actions: &MultiValueEncoded<T>) {
        require!(!actions.is_empty(), "No actions");
    }
}
