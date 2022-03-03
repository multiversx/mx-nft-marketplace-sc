elrond_wasm::imports!();

use crate::auction::*;

#[elrond_wasm::module]
pub trait ViewsModule: crate::storage::StorageModule {
    #[view(doesAuctionExist)]
    fn does_auction_exist(&self, auction_id: u64) -> bool {
        !self.auction_by_id(auction_id).is_empty()
    }

    #[view(getAuctionedToken)]
    fn get_auctioned_token(
        &self,
        auction_id: u64,
    ) -> OptionalValue<MultiValue3<TokenIdentifier, u64, BigUint>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();

            OptionalValue::Some(
                (
                    auction.auctioned_token.token_type,
                    auction.auctioned_token.nonce,
                    auction.nr_auctioned_tokens,
                )
                    .into(),
            )
        } else {
            OptionalValue::None
        }
    }

    #[endpoint(getAuctionType)]
    fn get_auction_type(&self, auction_id: u64) -> AuctionType {
        if self.does_auction_exist(auction_id) {
            self.auction_by_id(auction_id).get().auction_type
        } else {
            AuctionType::None
        }
    }

    #[view(getPaymentTokenForAuction)]
    fn get_payment_token_for_auction(
        &self,
        auction_id: u64,
    ) -> OptionalValue<MultiValue2<TokenIdentifier, u64>> {
        if self.does_auction_exist(auction_id) {
            let esdt_token = self.auction_by_id(auction_id).get().payment_token;

            OptionalValue::Some((esdt_token.token_type, esdt_token.nonce).into())
        } else {
            OptionalValue::None
        }
    }

    #[view(getMinMaxBid)]
    fn get_min_max_bid(&self, auction_id: u64) -> OptionalValue<MultiValue2<BigUint, BigUint>> {
        if self.does_auction_exist(auction_id) {
            let auction = self.auction_by_id(auction_id).get();

            OptionalValue::Some(
                (
                    auction.min_bid,
                    auction.max_bid.unwrap_or_else(|| BigUint::zero()),
                )
                    .into(),
            )
        } else {
            OptionalValue::None
        }
    }

    #[view(getStartTime)]
    fn get_start_time(&self, auction_id: u64) -> OptionalValue<u64> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().start_time)
        } else {
            OptionalValue::None
        }
    }

    #[view(getDeadline)]
    fn get_deadline(&self, auction_id: u64) -> OptionalValue<u64> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().deadline)
        } else {
            OptionalValue::None
        }
    }

    #[view(getOriginalOwner)]
    fn get_original_owner(&self, auction_id: u64) -> OptionalValue<ManagedAddress> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().original_owner)
        } else {
            OptionalValue::None
        }
    }

    #[view(getCurrentWinningBid)]
    fn get_current_winning_bid(&self, auction_id: u64) -> OptionalValue<BigUint> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().current_bid)
        } else {
            OptionalValue::None
        }
    }

    #[view(getCurrentWinner)]
    fn get_current_winner(&self, auction_id: u64) -> OptionalValue<ManagedAddress> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get().current_winner)
        } else {
            OptionalValue::None
        }
    }

    #[view(getFullAuctionData)]
    fn get_full_auction_data(&self, auction_id: u64) -> OptionalValue<Auction<Self::Api>> {
        if self.does_auction_exist(auction_id) {
            OptionalValue::Some(self.auction_by_id(auction_id).get())
        } else {
            OptionalValue::None
        }
    }
}
