#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod common;
pub mod external_proxies;
pub mod user_actions;

#[multiversx_sc::contract]
pub trait AccountAbstraction:
    common::users::UsersModule
    + common::signature::SignatureModule
    + user_actions::execution::ExecutionModule
    + user_actions::whitelist_actions::WhitelistActionsModule
    + common::external_sc_interactions::ExternalScInteractionsModule
    + common::custom_callbacks::CustomCallbacksModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
