use common_structs::PaymentsVec;

use crate::common::{custom_callbacks::CallbackProxy as _, signature::CheckExecutionSignatureArgs};

use crate::common::common_types::{
    ActionMultiValue, ActionStruct, CallType, EgldTxType, EsdtTxType, GasLimit, GeneralActionData,
    EGLD_TOKEN_ID,
};

const DEFAULT_EXTRA_CALLBACK_GAS: GasLimit = 10_000_000;
static INVALID_TX_DATA_ERR_MSG: &[u8] = b"Invalid Tx data";

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecutionModule:
    crate::common::users::UsersModule
    + crate::common::signature::SignatureModule
    + crate::common::custom_callbacks::CustomCallbacksModule
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
            let (mut action, nonce, signature) = (
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

            let egld_value = self.get_egld_value(&mut action.payments);
            self.execute_action_by_type(user_address.clone(), egld_value, action);

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

    fn build_egld_tx(
        &self,
        egld_value: BigUint,
        action_data: GeneralActionData<Self::Api>,
    ) -> EgldTxType<Self::Api> {
        require!(action_data.opt_execution.is_some(), INVALID_TX_DATA_ERR_MSG);

        let sc_exec_data = unsafe { action_data.opt_execution.unwrap_unchecked() };
        self.tx()
            .to(action_data.dest_address)
            .egld(egld_value)
            .raw_call(sc_exec_data.endpoint_name)
            .arguments_raw(sc_exec_data.args.into())
            .gas(sc_exec_data.gas_limit)
    }

    fn build_esdt_tx(&self, action_data: GeneralActionData<Self::Api>) -> EsdtTxType<Self::Api> {
        require!(action_data.opt_execution.is_some(), INVALID_TX_DATA_ERR_MSG);

        let sc_exec_data = unsafe { action_data.opt_execution.unwrap_unchecked() };
        self.tx()
            .to(action_data.dest_address)
            .multi_esdt(action_data.payments)
            .raw_call(sc_exec_data.endpoint_name)
            .arguments_raw(sc_exec_data.args.into())
            .gas(sc_exec_data.gas_limit)
    }

    fn get_egld_value(&self, payments: &mut PaymentsVec<Self::Api>) -> BigUint {
        let egld_token_id = TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID);
        let mut opt_egld_index = None;
        let mut egld_value = BigUint::zero();
        for (i, payment) in payments.iter().enumerate() {
            if payment.token_identifier != egld_token_id {
                continue;
            }

            require!(opt_egld_index.is_none(), "Only one EGLD payment allowed");

            opt_egld_index = Some(i);
            egld_value = payment.amount;
        }

        if let Some(index) = opt_egld_index {
            payments.remove(index);

            require!(payments.is_empty(), "Cannot transfer both EGLD and ESDT");
        }

        egld_value
    }

    fn execute_action_by_type(
        &self,
        user_address: ManagedAddress,
        egld_value: BigUint,
        action: GeneralActionData<Self::Api>,
    ) {
        match action.call_type {
            CallType::Transfer => {
                if egld_value == 0 {
                    self.tx()
                        .to(action.dest_address)
                        .multi_esdt(action.payments)
                        .transfer();
                } else {
                    self.tx()
                        .to(action.dest_address)
                        .egld(egld_value)
                        .transfer()
                }
            }
            CallType::Sync => {
                if egld_value == 0 {
                    let tx = self.build_esdt_tx(action);
                    tx.sync_call();
                } else {
                    let tx = self.build_egld_tx(egld_value, action);
                    tx.sync_call();
                }
            }
            CallType::Async => {
                let mut original_payments = action.payments.clone();
                if egld_value == 0 {
                    let tx = self.build_esdt_tx(action);
                    tx.with_callback(
                        self.callbacks()
                            .user_action_cb(user_address, original_payments),
                    )
                    .with_extra_gas_for_callback(DEFAULT_EXTRA_CALLBACK_GAS)
                    .register_promise();
                } else {
                    original_payments.push(EsdtTokenPayment::new(
                        TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID),
                        0,
                        egld_value.clone(),
                    ));

                    let tx = self.build_egld_tx(egld_value, action);
                    tx.with_callback(
                        self.callbacks()
                            .user_action_cb(user_address, original_payments),
                    )
                    .with_extra_gas_for_callback(DEFAULT_EXTRA_CALLBACK_GAS)
                    .register_promise();
                }
            }
        };
    }

    fn require_non_empty_actions<T>(&self, actions: &MultiValueEncoded<T>) {
        require!(!actions.is_empty(), "No actions");
    }
}
