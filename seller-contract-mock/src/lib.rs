#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::derive::contract]
pub trait Adder {
    #[init]
    fn init(&self) {}

    #[endpoint]
    fn claim(
        &self,
        marketplace_sc_address: ManagedAddress,
        token_id: TokenIdentifier,
        token_nonce: u64,
    ) {
        let caller = self.blockchain().get_caller();

        self.market_proxy(marketplace_sc_address)
            .claim_tokens(token_id, token_nonce, caller)
            .execute_on_dest_context();
    }

    #[proxy]
    fn market_proxy(&self, sc_address: ManagedAddress) -> esdt_nft_marketplace::Proxy<Self::Api>;
}
