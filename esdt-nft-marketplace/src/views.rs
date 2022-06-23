elrond_wasm::imports!();

use crate::auction::*;

#[elrond_wasm::module]
pub trait ViewsModule:
    crate::auction::AuctionModule
    + crate::token_distribution::TokenDistributionModule
    + crate::events::EventsModule
    + crate::common_util_functions::CommonUtilFunctions
{
    #[view(doesAuctionExist)]
    fn does_auction_exist(&self, auction_id: u64) -> bool {
        !self.auction_by_id(auction_id).is_empty()
    }

    #[view(getAuctionedToken)]
    fn get_auctioned_token(
        &self,
        auction_id: u64,
    ) -> OptionalValue<MultiValue3<TokenIdentifier, u64, BigUint>> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            let auction = auction_mapper.get();

            OptionalValue::Some(
                (
                    auction.auctioned_tokens.token_identifier,
                    auction.auctioned_tokens.token_nonce,
                    auction.auctioned_tokens.amount,
                )
                    .into(),
            )
        } else {
            OptionalValue::None
        }
    }

    #[endpoint(getAuctionType)]
    fn get_auction_type(&self, auction_id: u64) -> AuctionType {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            auction_mapper.get().auction_type
        } else {
            AuctionType::None
        }
    }

    #[view(getPaymentTokenForAuction)]
    fn get_payment_token_for_auction(
        &self,
        auction_id: u64,
    ) -> OptionalValue<MultiValue2<EgldOrEsdtTokenIdentifier, u64>> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            let auction = auction_mapper.get();

            OptionalValue::Some((auction.payment_token, auction.payment_nonce).into())
        } else {
            OptionalValue::None
        }
    }

    #[view(getMinMaxBid)]
    fn get_min_max_bid(&self, auction_id: u64) -> OptionalValue<MultiValue2<BigUint, BigUint>> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            let auction = auction_mapper.get();

            OptionalValue::Some((auction.min_bid, auction.max_bid.unwrap_or_default()).into())
        } else {
            OptionalValue::None
        }
    }

    #[view(getStartTime)]
    fn get_start_time(&self, auction_id: u64) -> OptionalValue<u64> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            OptionalValue::Some(auction_mapper.get().start_time)
        } else {
            OptionalValue::None
        }
    }

    #[view(getDeadline)]
    fn get_deadline(&self, auction_id: u64) -> OptionalValue<u64> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            OptionalValue::Some(auction_mapper.get().deadline)
        } else {
            OptionalValue::None
        }
    }

    #[view(getOriginalOwner)]
    fn get_original_owner(&self, auction_id: u64) -> OptionalValue<ManagedAddress> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            OptionalValue::Some(auction_mapper.get().original_owner)
        } else {
            OptionalValue::None
        }
    }

    #[view(getCurrentWinningBid)]
    fn get_current_winning_bid(&self, auction_id: u64) -> OptionalValue<BigUint> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            OptionalValue::Some(auction_mapper.get().current_bid)
        } else {
            OptionalValue::None
        }
    }

    #[view(getCurrentWinner)]
    fn get_current_winner(&self, auction_id: u64) -> OptionalValue<ManagedAddress> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            OptionalValue::Some(auction_mapper.get().current_winner)
        } else {
            OptionalValue::None
        }
    }

    #[view(getFullAuctionData)]
    fn get_full_auction_data(&self, auction_id: u64) -> OptionalValue<Auction<Self::Api>> {
        let auction_mapper = self.auction_by_id(auction_id);
        if !auction_mapper.is_empty() {
            OptionalValue::Some(auction_mapper.get())
        } else {
            OptionalValue::None
        }
    }
}
