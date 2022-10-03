elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::offer::Offer;
use super::auction::{Auction, AuctionType};

#[allow(clippy::too_many_arguments)]
#[elrond_wasm::module]
pub trait EventsModule {
    fn emit_auction_token_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.auction_token_event(
            &auction.auctioned_tokens.token_identifier,
            auction.auctioned_tokens.token_nonce,
            auction_id,
            &auction.auctioned_tokens.amount,
            &auction.original_owner,
            &auction.min_bid,
            &auction.max_bid.unwrap_or_else(BigUint::zero),
            auction.start_time,
            auction.deadline,
            auction.payment_token,
            auction.payment_nonce,
            auction.auction_type,
            auction.creator_royalties_percentage,
        )
    }

    fn emit_bid_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.bid_event(
            &auction.auctioned_tokens.token_identifier,
            auction.auctioned_tokens.token_nonce,
            auction_id,
            &auction.auctioned_tokens.amount,
            &auction.current_winner,
            &auction.current_bid,
        );
    }

    fn emit_end_auction_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.end_auction_event(
            &auction.auctioned_tokens.token_identifier,
            auction.auctioned_tokens.token_nonce,
            auction_id,
            &auction.auctioned_tokens.amount,
            &auction.current_winner,
            &auction.current_bid,
        );
    }

    fn emit_buy_sft_event(
        self,
        auction_id: u64,
        auction: Auction<Self::Api>,
        nr_bought_tokens: BigUint,
    ) {
        self.buy_sft_event(
            &auction.auctioned_tokens.token_identifier,
            auction.auctioned_tokens.token_nonce,
            auction_id,
            &nr_bought_tokens,
            &auction.current_winner,
            &auction.min_bid,
        );
    }

    fn emit_withdraw_event(self, auction_id: u64, auction: Auction<Self::Api>) {
        self.withdraw_event(
            &auction.auctioned_tokens.token_identifier,
            auction.auctioned_tokens.token_nonce,
            auction_id,
            &auction.auctioned_tokens.amount,
            &auction.original_owner,
        );
    }

    fn emit_offer_token_event(self, offer_id: u64, offer: Offer<Self::Api>) {
        self.offer_token_event(
            offer_id,
            &offer.offer_token.token_identifier,
            offer.offer_token.token_nonce,
            &offer.offer_token.amount,
            &offer.payment.token_identifier,
            offer.payment.token_nonce,
            &offer.payment.amount,
            &offer.offer_owner,
            offer.start_time,
            offer.deadline,
        )
    }

    fn emit_withdraw_offer_event(self, offer_id: u64, offer: Offer<Self::Api>) {
        self.withdraw_offer_token_event(
            offer_id,
            &offer.offer_token.token_identifier,
            offer.offer_token.token_nonce,
            &offer.offer_token.amount,
            &offer.payment.token_identifier,
            offer.payment.token_nonce,
            &offer.payment.amount,
            &offer.offer_owner,
            offer.start_time,
            offer.deadline,
        )
    }

    fn emit_accept_offer_event(
        self,
        offer_id: u64,
        offer: Offer<Self::Api>,
        seller: &ManagedAddress,
    ) {
        self.accept_offer_token_event(
            offer_id,
            &offer.offer_token.token_identifier,
            offer.offer_token.token_nonce,
            &offer.offer_token.amount,
            &offer.payment.token_identifier,
            offer.payment.token_nonce,
            &offer.payment.amount,
            &offer.offer_owner,
            seller,
            offer.start_time,
            offer.deadline,
        )
    }

    #[event("auction_token_event")]
    fn auction_token_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] auctioned_token_amount: &BigUint,
        #[indexed] seller: &ManagedAddress,
        #[indexed] min_bid: &BigUint,
        #[indexed] max_bid: &BigUint,
        #[indexed] start_time: u64,
        #[indexed] deadline: u64,
        #[indexed] accepted_payment_token: EgldOrEsdtTokenIdentifier,
        #[indexed] accepted_payment_token_nonce: u64,
        #[indexed] auction_type: AuctionType,
        creator_royalties_percentage: BigUint, // between 0 and 10,000
    );

    #[event("bid_event")]
    fn bid_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_auctioned_tokens: &BigUint,
        #[indexed] bidder: &ManagedAddress,
        #[indexed] bid_amount: &BigUint,
    );

    #[event("end_auction_event")]
    fn end_auction_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_auctioned_tokens: &BigUint,
        #[indexed] auction_winner: &ManagedAddress,
        #[indexed] winning_bid_amount: &BigUint,
    );

    #[event("buy_sft_event")]
    fn buy_sft_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_bought_tokens: &BigUint,
        #[indexed] buyer: &ManagedAddress,
        #[indexed] bid_sft_amount: &BigUint,
    );

    #[event("withdraw_event")]
    fn withdraw_event(
        &self,
        #[indexed] auction_token_id: &TokenIdentifier,
        #[indexed] auctioned_token_nonce: u64,
        #[indexed] auction_id: u64,
        #[indexed] nr_auctioned_tokens: &BigUint,
        #[indexed] seller: &ManagedAddress,
    );

    #[event("offer_token_event")]
    fn offer_token_event(
        &self,
        #[indexed] offer_id: u64,
        #[indexed] offer_token_id: &TokenIdentifier,
        #[indexed] offer_token_nonce: u64,
        #[indexed] offer_amount: &BigUint,
        #[indexed] payment_token_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] payment_amount: &BigUint,
        #[indexed] buyer: &ManagedAddress,
        #[indexed] start_time: u64,
        #[indexed] deadline: u64,
    );

    #[event("withdraw_offer_token_event")]
    fn withdraw_offer_token_event(
        &self,
        #[indexed] offer_id: u64,
        #[indexed] offer_token_id: &TokenIdentifier,
        #[indexed] offer_token_nonce: u64,
        #[indexed] offer_amount: &BigUint,
        #[indexed] payment_token_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] payment_amount: &BigUint,
        #[indexed] buyer: &ManagedAddress,
        #[indexed] start_time: u64,
        #[indexed] deadline: u64,
    );

    #[event("accept_offer_token_event")]
    fn accept_offer_token_event(
        &self,
        #[indexed] offer_id: u64,
        #[indexed] offer_token_id: &TokenIdentifier,
        #[indexed] offer_token_nonce: u64,
        #[indexed] offer_amount: &BigUint,
        #[indexed] payment_token_type: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_token_nonce: u64,
        #[indexed] payment_amount: &BigUint,
        #[indexed] buyer: &ManagedAddress,
        #[indexed] seller: &ManagedAddress,
        #[indexed] start_time: u64,
        #[indexed] deadline: u64,
    );
}
