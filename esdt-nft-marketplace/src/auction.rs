elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
pub const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Auction<M: ManagedTypeApi> {
    pub auctioned_tokens: EsdtTokenPayment<M>,
    pub auction_type: AuctionType,

    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub min_bid: BigUint<M>,
    pub max_bid: Option<BigUint<M>>,
    pub min_bid_diff: BigUint<M>,
    pub start_time: u64,
    pub deadline: u64,

    pub original_owner: ManagedAddress<M>,
    pub current_bid: BigUint<M>,
    pub current_winner: ManagedAddress<M>,
    pub marketplace_cut_percentage: BigUint<M>,
    pub creator_royalties_percentage: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum AuctionType {
    None,
    Nft,
    SftAll,
    SftOnePerPayment,
}

#[elrond_wasm::module]
pub trait AuctionModule:
    crate::token_distribution::TokenDistributionModule
    + crate::events::EventsModule
    + crate::common_util_functions::CommonUtilFunctions
{
    #[payable("*")]
    #[endpoint(auctionToken)]
    #[allow(clippy::too_many_arguments)]
    fn auction_token(
        &self,
        min_bid: BigUint,
        max_bid: BigUint,
        deadline: u64,
        accepted_payment_token: EgldOrEsdtTokenIdentifier,
        opt_min_bid_diff: OptionalValue<BigUint>,
        opt_sft_max_one_per_payment: OptionalValue<bool>,
        opt_accepted_payment_token_nonce: OptionalValue<u64>,
        opt_start_time: OptionalValue<u64>,
    ) -> u64 {
        let (nft_type, nft_nonce, nft_amount) = self.call_value().single_esdt().into_tuple();

        let current_time = self.blockchain().get_block_timestamp();
        let start_time = match opt_start_time {
            OptionalValue::Some(0) => current_time,
            OptionalValue::Some(st) => st,
            OptionalValue::None => current_time,
        };
        let sft_max_one_per_payment = opt_sft_max_one_per_payment
            .into_option()
            .unwrap_or_default();

        if sft_max_one_per_payment {
            require!(
                min_bid == max_bid,
                "Price must be fixed for this type of auction (min bid equal to max bid)"
            );
        }

        let opt_max_bid = if max_bid > 0u32 {
            require!(min_bid <= max_bid, "Min bid can't be higher than max bid");

            Some(max_bid)
        } else {
            None
        };

        require!(min_bid > 0, "Min bid must be higher than 0");
        require!(
            nft_nonce > 0,
            "Only Semi-Fungible and Non-Fungible tokens can be auctioned"
        );
        require!(deadline > current_time, "Deadline can't be in the past");
        require!(
            start_time >= current_time && start_time < deadline,
            "Invalid start time"
        );

        let marketplace_cut_percentage = self.bid_cut_percentage().get();
        let creator_royalties_percentage = self.get_nft_info(&nft_type, nft_nonce).royalties;

        require!(
            &marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );

        let min_bid_diff = match opt_min_bid_diff {
            OptionalValue::Some(min_diff) => min_diff,
            OptionalValue::None => BigUint::zero(),
        };

        let accepted_payment_nft_nonce = if accepted_payment_token.is_egld() {
            0
        } else {
            opt_accepted_payment_token_nonce
                .into_option()
                .unwrap_or_default()
        };

        let auction_id = self.last_valid_auction_id().get() + 1;
        self.last_valid_auction_id().set(&auction_id);

        let auction_type = if nft_amount > NFT_AMOUNT {
            match sft_max_one_per_payment {
                true => AuctionType::SftOnePerPayment,
                false => AuctionType::SftAll,
            }
        } else {
            AuctionType::Nft
        };

        let auction = Auction {
            auctioned_tokens: EsdtTokenPayment::new(nft_type, nft_nonce, nft_amount),
            auction_type,
            payment_token: accepted_payment_token,
            payment_nonce: accepted_payment_nft_nonce,
            min_bid,
            max_bid: opt_max_bid,
            min_bid_diff,
            start_time,
            deadline,

            original_owner: self.blockchain().get_caller(),
            current_bid: BigUint::zero(),
            current_winner: ManagedAddress::zero(),
            marketplace_cut_percentage,
            creator_royalties_percentage,
        };
        self.auction_by_id(auction_id).set(&auction);

        self.emit_auction_token_event(auction_id, auction);

        auction_id
    }

    #[endpoint(endAuction)]
    fn end_auction(&self, auction_id: u64) {
        let auction = self.try_get_auction(auction_id);
        let current_time = self.blockchain().get_block_timestamp();

        let deadline_reached = current_time > auction.deadline;
        let max_bid_reached = if let Some(max_bid) = &auction.max_bid {
            &auction.current_bid == max_bid
        } else {
            false
        };

        require!(
            deadline_reached || max_bid_reached,
            "Auction deadline has not passed nor is the current bid equal to max bid"
        );
        require!(
            auction.auction_type != AuctionType::SftOnePerPayment,
            "Cannot end this type of auction"
        );

        self.distribute_tokens_after_auction_end(&auction, None);
        self.auction_by_id(auction_id).clear();

        self.emit_end_auction_event(auction_id, auction);
    }

    #[endpoint]
    fn withdraw(&self, auction_id: u64) {
        let auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();

        require!(
            auction.original_owner == caller,
            "Only the original owner can withdraw"
        );
        require!(
            auction.current_bid == 0 || auction.auction_type == AuctionType::SftOnePerPayment,
            "Can't withdraw, NFT already has bids"
        );

        self.auction_by_id(auction_id).clear();

        let nft_type = &auction.auctioned_tokens.token_identifier;
        let nft_nonce = auction.auctioned_tokens.token_nonce;
        let nft_amount = &auction.auctioned_tokens.amount;
        self.transfer_or_save_payment(
            &caller,
            &EgldOrEsdtTokenIdentifier::esdt(nft_type.clone()),
            nft_nonce,
            nft_amount,
        );

        self.emit_withdraw_event(auction_id, auction);
    }

    fn try_get_auction(&self, auction_id: u64) -> Auction<Self::Api> {
        let auction_mapper = self.auction_by_id(auction_id);
        require!(!auction_mapper.is_empty(), "Auction does not exist");
        auction_mapper.get()
    }

    #[storage_mapper("auctionById")]
    fn auction_by_id(&self, auction_id: u64) -> SingleValueMapper<Auction<Self::Api>>;

    #[view(getLastValidAuctionId)]
    #[storage_mapper("lastValidAuctionId")]
    fn last_valid_auction_id(&self) -> SingleValueMapper<u64>;

    #[view(getMarketplaceCutPercentage)]
    #[storage_mapper("bidCutPercentage")]
    fn bid_cut_percentage(&self) -> SingleValueMapper<BigUint>;
}
