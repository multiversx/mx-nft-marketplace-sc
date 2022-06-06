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
        token_id: EgldOrEsdtTokenIdentifier,
        token_nonce: u64,
    ) {
        let caller = self.blockchain().get_caller();
        let mut token_nonce_pairs = MultiValueEncoded::new();
        token_nonce_pairs.push(MultiValue2::from((token_id, token_nonce)));

        self.market_proxy(marketplace_sc_address)
            .claim_tokens(caller, token_nonce_pairs)
            .execute_on_dest_context_ignore_result();
    }

    #[proxy]
    fn market_proxy(&self, sc_address: ManagedAddress) -> esdt_nft_marketplace::Proxy<Self::Api>;
}
