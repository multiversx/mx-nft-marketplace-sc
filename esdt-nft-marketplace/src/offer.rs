elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::auction::EsdtToken;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct Offer<M: ManagedTypeApi> {
    pub offer_token: EsdtToken<M>,
    pub payment_token: EsdtToken<M>,
    pub offer_price: BigUint<M>,
    pub start_time: u64,
    pub deadline: u64,
    pub offer_owner: ManagedAddress<M>,
    pub marketplace_cut_percentage: BigUint<M>,
    pub creator_royalties_percentage: BigUint<M>,
}
