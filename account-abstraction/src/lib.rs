#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod common;
pub mod user_actions;

#[multiversx_sc::contract]
pub trait AccountAbstraction:
    common::users::UsersModule
    + common::signature::SignatureModule
    + user_actions::execution::ExecutionModule
    + user_actions::whitelist_actions::WhitelistActionsModule
    + common::custom_callbacks::CustomCallbacksModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
