#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod common;
pub mod own_proxy;
pub mod user_actions;

#[multiversx_sc::contract]
pub trait AccountAbstraction:
    common::users::UsersModule
    + common::signature::SignatureModule
    + user_actions::execution::ExecutionModule
    + user_actions::whitelist_actions::WhitelistActionsModule
    + user_actions::intents::IntentsModule
    + user_actions::intent_storage::IntentStorageModule
    + user_actions::views::ViewsModule
    + common::custom_callbacks::CustomCallbacksModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
