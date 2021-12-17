elrond_wasm::imports!();

use crate::auction::*;

#[elrond_wasm::module]
pub trait StorageModule {
    #[view(getMarketplaceCutPercentage)]
    #[storage_mapper("bidCutPercentage")]
    fn bid_cut_percentage(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("auctionById")]
    fn auction_by_id(&self, auction_id: u64) -> SingleValueMapper<Auction<Self::Api>>;

    #[view(getLastValidAuctionId)]
    #[storage_mapper("lastValidAuctionId")]
    fn last_valid_auction_id(&self) -> SingleValueMapper<u64>;

    #[view(getClaimableAmount)]
    #[storage_mapper("claimableAmount")]
    fn claimable_amount(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
        token_nonce: u64,
    ) -> SingleValueMapper<BigUint>;
}
