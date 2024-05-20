#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait AccountAbstraction {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
