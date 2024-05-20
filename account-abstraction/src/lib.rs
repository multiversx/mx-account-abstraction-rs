#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod unique_payments;
pub mod users;

#[multiversx_sc::contract]
pub trait AccountAbstraction: users::UsersModule + utils::UtilsModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
