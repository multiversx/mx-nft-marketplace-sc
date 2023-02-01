multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CommonUtilFunctions: multiversx_sc_modules::pause::PauseModule {
    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }
}
