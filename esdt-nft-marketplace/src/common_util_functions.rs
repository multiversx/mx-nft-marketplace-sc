elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait CommonUtilFunctions: elrond_wasm_modules::pause::PauseModule {
    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }

    fn require_not_paused(&self) {
        require!(self.not_paused(), "Marketplace is paused");
    }
}
