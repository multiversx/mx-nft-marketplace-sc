elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait TokenWhitelistModule {
    #[only_owner]
    #[endpoint(addTokensToWhitelist)]
    fn add_tokens_to_whitelist(&self, tokens_to_add: MultiValueEncoded<EgldOrEsdtTokenIdentifier>) {
        let mut whitelisted_tokens_mapper = self.whitelisted_tokens();
        for token_id in tokens_to_add {
            require!(token_id.is_valid(), "Whitelisted token is not valid");
            let _ = whitelisted_tokens_mapper.insert(token_id);
        }
    }

    #[only_owner]
    #[endpoint(removeTokensFromWhitelist)]
    fn remove_tokens_from_whitelist(
        &self,
        tokens_to_remove: MultiValueEncoded<EgldOrEsdtTokenIdentifier>,
    ) {
        let mut whitelisted_tokens_mapper = self.whitelisted_tokens();
        for token_id in tokens_to_remove {
            let _ = whitelisted_tokens_mapper.swap_remove(&token_id);
        }
    }

    fn require_token_whitelisted(&self, token_id: &EgldOrEsdtTokenIdentifier) {
        let whitelisted_tokens_mapper = self.whitelisted_tokens();
        if !whitelisted_tokens_mapper.is_empty() {
            require!(
                self.whitelisted_tokens().contains(token_id),
                "Token is not whitelisted"
            );
        }
    }

    #[view(getWhitelistedTokens)]
    #[storage_mapper("whitelistedTokens")]
    fn whitelisted_tokens(&self) -> UnorderedSetMapper<EgldOrEsdtTokenIdentifier>;
}
