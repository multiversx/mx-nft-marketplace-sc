elrond_wasm::imports!();

use crate::{auction::*, offer::Offer};

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

    #[view(getAuctionsByAddress)]
    #[storage_mapper("auctionsByAddress")]
    fn auctions_by_address(&self, address: &ManagedAddress) -> SetMapper<u64>;

    #[view(getAuctionsByToken)]
    #[storage_mapper("auctionsByToken")]
    fn auctions_by_token(&self, token_id: &TokenIdentifier, token_nonce: u64) -> SetMapper<u64>;

    #[view(getLastValidOfferId)]
    #[storage_mapper("lastValidOfferId")]
    fn last_valid_offer_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("offerById")]
    fn offer_by_id(&self, offer_id: u64) -> SingleValueMapper<Offer<Self::Api>>;

    #[view(getOffersByAddress)]
    #[storage_mapper("offersByAddress")]
    fn offers_by_address(&self, address: &ManagedAddress) -> SetMapper<u64>;

    #[view(getOffersByToken)]
    #[storage_mapper("offersByToken")]
    fn offers_by_token(&self, token_id: &TokenIdentifier, token_nonce: u64) -> SetMapper<u64>;

    #[view(getOfferExists)]
    #[storage_mapper("offerExists")]
    fn offer_exists(
        &self,
        address: &ManagedAddress,
        nft: &TokenIdentifier,
        nonce: u64,
        payment_token: &TokenIdentifier,
    ) -> SingleValueMapper<bool>;
}
