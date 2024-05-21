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
    + common::custom_callbacks::CustomCallbacksModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
