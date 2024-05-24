use crate::common::common_types::{
    Action, ActionMultiValue, ActionStruct, CallType, GeneralActionData,
};

pub type IntentId = u64;

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode)]
pub enum IntentType {
    AwaitingExecution,
    InProgress,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode)]
pub struct Intent<M: ManagedTypeApi> {
    pub intent_type: IntentType,
    pub intent_data: GeneralActionData<M>,
}

impl<M: ManagedTypeApi> Intent<M> {
    #[inline]
    pub fn new(intent_type: IntentType, intent_data: GeneralActionData<M>) -> Self {
        Self {
            intent_type,
            intent_data,
        }
    }
}

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait IntentsModule:
    crate::common::users::UsersModule
    + crate::common::signature::SignatureModule
    + crate::common::custom_callbacks::CustomCallbacksModule
    + super::execution::ExecutionModule
    + super::intent_storage::IntentStorageModule
{
    #[endpoint(saveIntents)]
    fn save_intents(
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
        self.save_intents_common(&user_address, &actions_vec, &own_sc_address);
    }

    /// Pairs of (user_address, actions_vec)
    #[endpoint(multiUserSaveIntents)]
    fn multi_user_save_intents(
        &self,
        args: MultiValueEncoded<MultiValue2<ManagedAddress, ManagedVec<ActionStruct<Self::Api>>>>,
    ) {
        self.require_non_empty_actions(&args);

        let own_sc_address = self.blockchain().get_sc_address();
        for pair in args {
            let (user_address, actions_vec) = pair.into_tuple();
            self.save_intents_common(&user_address, &actions_vec, &own_sc_address);
        }
    }

    #[endpoint(executeIntent)]
    fn execute_intent(&self, user_address: ManagedAddress, intent_id: IntentId) {
        let user_id = self.user_ids().get_id_non_zero(&user_address);
        let intent_mapper = self.user_intent(user_id, intent_id);
        require!(!intent_mapper.is_empty(), "Intent doesn't exist");

        let mut intent = intent_mapper.get();
        require!(
            matches!(intent.intent_type, IntentType::AwaitingExecution),
            "Intent execution already in progress"
        );

        let egld_value = self.get_egld_value(&mut intent.intent_data.payments);
        self.execute_action_by_type(
            user_address.clone(),
            egld_value,
            intent.intent_data,
            Some(intent_id),
        );
    }

    fn save_intents_common(
        &self,
        user_address: &ManagedAddress,
        actions: &ManagedVec<ActionStruct<Self::Api>>,
        own_sc_address: &ManagedAddress,
    ) {
        self.check_can_execute_actions(user_address, actions, own_sc_address);

        let user_id = self.user_ids().get_id(user_address);
        let mut intent_id = self.last_intent_id().get() + 1;
        let mut all_intents_mapper = self.all_user_intents(user_id);
        for action_struct in actions {
            let action = action_struct.get_general_action_data();
            require!(
                matches!(action.call_type, CallType::Async),
                "Only async call supported"
            );

            let _ = all_intents_mapper.insert(intent_id);
            self.user_intent(user_id, intent_id)
                .set(Intent::new(IntentType::AwaitingExecution, action));

            intent_id += 1;
        }

        self.last_intent_id().set(intent_id);
    }
}
