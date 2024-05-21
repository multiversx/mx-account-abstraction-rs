#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod signature;
pub mod unique_payments;
pub mod user_actions;
pub mod users;

#[multiversx_sc::contract]
pub trait AccountAbstraction:
    users::UsersModule
    + signature::SignatureModule
    + user_actions::execution::ExecutionModule
    + user_actions::custom_callbacks::CustomCallbacksModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
