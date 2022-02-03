#![no_std]

elrond_wasm::imports!();

pub mod auction;
use auction::*;

mod events;
mod storage;
mod views;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1; // Token has to be unique to be considered NFT

#[elrond_wasm::contract]
pub trait EsdtNftMarketplace:
    storage::StorageModule + views::ViewsModule + events::EventsModule
{
    #[init]
    fn init(&self, bid_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
    }

    // endpoints - owner-only

    #[only_owner]
    #[endpoint(setCutPercentage)]
    fn set_percentage_cut(&self, new_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(new_cut_percentage);
    }

    // endpoints

    #[payable("*")]
    #[endpoint(auctionToken)]
    #[allow(clippy::too_many_arguments)]
    fn auction_token(
        &self,
        #[payment_token] nft_type: TokenIdentifier,
        #[payment_nonce] nft_nonce: u64,
        #[payment_amount] nft_amount: BigUint,
        min_bid: BigUint,
        max_bid: BigUint,
        deadline: u64,
        accepted_payment_token: TokenIdentifier,
        #[var_args] opt_accepted_payment_token_nonce: OptionalArg<u64>,
        #[var_args] opt_sft_max_one_per_payment: OptionalArg<bool>,
        #[var_args] opt_start_time: OptionalArg<u64>,
    ) -> u64 {
        require!(
            nft_amount >= BigUint::from(NFT_AMOUNT),
            "Must tranfer at least one"
        );

        let current_time = self.blockchain().get_block_timestamp();
        let start_time = match opt_start_time {
            OptionalArg::Some(st) => st,
            OptionalArg::None => current_time,
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
            require!(min_bid <= max_bid, "Min bid can't higher than max bid");

            Some(max_bid)
        } else {
            None
        };

        require!(min_bid > 0u32, "Min bid must be higher than 0");
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

        let accepted_payment_nft_nonce = if accepted_payment_token.is_egld() {
            0
        } else {
            opt_accepted_payment_token_nonce
                .into_option()
                .unwrap_or_default()
        };

        let auction_id = self.last_valid_auction_id().get() + 1;
        self.last_valid_auction_id().set(&auction_id);

        let auction_type = if nft_amount > BigUint::from(NFT_AMOUNT) {
            match sft_max_one_per_payment {
                true => AuctionType::SftOnePerPayment,
                false => AuctionType::SftAll,
            }
        } else {
            AuctionType::Nft
        };

        let auction = Auction {
            auctioned_token: EsdtToken {
                token_type: nft_type,
                nonce: nft_nonce,
            },
            nr_auctioned_tokens: nft_amount,
            auction_type,

            payment_token: EsdtToken {
                token_type: accepted_payment_token,
                nonce: accepted_payment_nft_nonce,
            },
            min_bid,
            max_bid: opt_max_bid,
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

    #[payable("*")]
    #[endpoint]
    fn bid(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
    ) {
        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            auction.auction_type != AuctionType::SftOnePerPayment,
            "Cannot bid on this type of auction"
        );
        require!(
            auction.auctioned_token.token_type == nft_type
                && auction.auctioned_token.nonce == nft_nonce,
            "Auction ID does not match the token"
        );
        require!(
            auction.original_owner != caller,
            "Can't bid on your own token"
        );
        require!(
            current_time >= auction.start_time,
            "Auction hasn't started yet"
        );
        require!(current_time < auction.deadline, "Auction ended already");
        require!(
            payment_token == auction.payment_token.token_type
                && payment_token_nonce == auction.payment_token.nonce,
            "Wrong token used as payment"
        );
        require!(auction.current_winner != caller, "Can't outbid yourself");
        require!(
            payment_amount >= auction.min_bid,
            "Bid must be higher than or equal to the min bid"
        );
        require!(
            payment_amount > auction.current_bid,
            "Bid must be higher than the current winning bid"
        );

        if let Some(max_bid) = &auction.max_bid {
            require!(
                &payment_amount <= max_bid,
                "Bid must be less than or equal to the max bid"
            );
        }

        // refund losing bid
        if auction.current_winner != ManagedAddress::zero() {
            self.transfer_or_save_payment(
                &auction.current_winner,
                &auction.payment_token.token_type,
                auction.payment_token.nonce,
                &auction.current_bid,
                b"bid refund",
            );
        }

        // update auction bid and winner
        auction.current_bid = payment_amount;
        auction.current_winner = caller;
        self.auction_by_id(auction_id).set(&auction);

        self.emit_bid_event(auction_id, auction);
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

    #[allow(clippy::too_many_arguments)]
    #[payable("*")]
    #[endpoint(buySft)]
    fn buy_sft(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        #[var_args] opt_sft_buy_amount: OptionalArg<BigUint>,
    ) {
        let mut auction = self.try_get_auction(auction_id);
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();

        let sft_buy_amount = match opt_sft_buy_amount {
            OptionalArg::Some(amt) => amt,
            OptionalArg::None => BigUint::from(NFT_AMOUNT),
        };
        let sft_total_value = &sft_buy_amount * &auction.min_bid;

        require!(sft_buy_amount > 0, "Must by more than 0");
        require!(
            auction.auction_type == AuctionType::SftOnePerPayment,
            "Cannot buy SFT for this type of auction"
        );
        require!(
            auction.auctioned_token.token_type == nft_type
                && auction.auctioned_token.nonce == nft_nonce,
            "Auction ID does not match the token"
        );
        require!(auction.original_owner != caller, "Can't buy your own token");
        require!(
            sft_buy_amount <= auction.nr_auctioned_tokens,
            "Not enough SFTs available"
        );
        require!(
            payment_token == auction.payment_token.token_type
                && payment_token_nonce == auction.payment_token.nonce,
            "Wrong token used as payment"
        );
        require!(
            sft_total_value == payment_amount,
            "Wrong amount paid, must pay equal to the selling price"
        );
        require!(
            current_time >= auction.start_time,
            "Cannot buy SFT before start time"
        );
        require!(
            current_time <= auction.deadline,
            "Cannot buy SFT after deadline"
        );

        auction.current_winner = caller;
        auction.current_bid = payment_amount;
        self.distribute_tokens_after_auction_end(&auction, Some(&sft_buy_amount));

        auction.nr_auctioned_tokens -= &sft_buy_amount;
        if auction.nr_auctioned_tokens == 0 {
            self.auction_by_id(auction_id).clear();
        } else {
            self.auction_by_id(auction_id).set(&auction);
        }

        self.emit_buy_sft_event(auction_id, auction, sft_buy_amount);
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

        let nft_type = &auction.auctioned_token.token_type;
        let nft_nonce = auction.auctioned_token.nonce;
        let nft_amount = &auction.nr_auctioned_tokens;
        self.transfer_or_save_payment(&caller, nft_type, nft_nonce, nft_amount, b"returned token");

        self.emit_withdraw_event(auction_id, auction);
    }

    #[endpoint(claimTokens)]
    fn claim_tokens(
        &self,
        token_id: TokenIdentifier,
        token_nonce: u64,
        claim_destination: ManagedAddress,
    ) {
        let caller = self.blockchain().get_caller();
        let amount_mapper = self.claimable_amount(&caller, &token_id, token_nonce);
        let amount = amount_mapper.get();

        if amount > 0 {
            amount_mapper.clear();

            self.send()
                .direct(&claim_destination, &token_id, token_nonce, &amount, &[]);
        }
    }

    // private

    fn try_get_auction(&self, auction_id: u64) -> Auction<Self::Api> {
        require!(
            self.does_auction_exist(auction_id),
            "Auction does not exist"
        );
        self.auction_by_id(auction_id).get()
    }

    fn calculate_cut_amount(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / PERCENTAGE_TOTAL
    }

    fn calculate_winning_bid_split(
        &self,
        auction: &Auction<Self::Api>,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&auction.current_bid, &auction.creator_royalties_percentage);
        let bid_cut_amount =
            self.calculate_cut_amount(&auction.current_bid, &auction.marketplace_cut_percentage);
        let mut seller_amount_to_send = auction.current_bid.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &bid_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: bid_cut_amount,
            seller: seller_amount_to_send,
        }
    }

    fn distribute_tokens_after_auction_end(
        &self,
        auction: &Auction<Self::Api>,
        opt_sft_amount: Option<&BigUint>,
    ) {
        let nft_type = &auction.auctioned_token.token_type;
        let nft_nonce = auction.auctioned_token.nonce;

        if !auction.current_winner.is_zero() {
            let nft_info = self.get_nft_info(nft_type, nft_nonce);
            let token_id = &auction.payment_token.token_type;
            let nonce = auction.payment_token.nonce;
            let bid_split_amounts = self.calculate_winning_bid_split(auction);

            // send part as cut for contract owner
            let owner = self.blockchain().get_owner_address();
            self.transfer_or_save_payment(
                &owner,
                token_id,
                nonce,
                &bid_split_amounts.marketplace,
                b"bid cut for sold token",
            );

            // send part as royalties to creator
            self.transfer_or_save_payment(
                &nft_info.creator,
                token_id,
                nonce,
                &bid_split_amounts.creator,
                b"royalties for sold token",
            );

            // send rest of the bid to original owner
            self.transfer_or_save_payment(
                &auction.original_owner,
                token_id,
                nonce,
                &bid_split_amounts.seller,
                b"sold token",
            );

            // send NFT to auction winner
            let nft_amount = BigUint::from(NFT_AMOUNT);
            let nft_amount_to_send = match auction.auction_type {
                AuctionType::Nft => &nft_amount,
                AuctionType::SftOnePerPayment => match opt_sft_amount {
                    Some(amt) => amt,
                    None => &nft_amount,
                },
                _ => &auction.nr_auctioned_tokens,
            };
            self.transfer_or_save_payment(
                &auction.current_winner,
                nft_type,
                nft_nonce,
                nft_amount_to_send,
                b"bought token at auction",
            );
        } else {
            // return to original owner
            self.transfer_or_save_payment(
                &auction.original_owner,
                nft_type,
                nft_nonce,
                &auction.nr_auctioned_tokens,
                b"returned token",
            );
        }
    }

    fn transfer_or_save_payment(
        &self,
        to: &ManagedAddress,
        token_id: &TokenIdentifier,
        nonce: u64,
        amount: &BigUint,
        data: &'static [u8],
    ) {
        if self.blockchain().is_smart_contract(to) {
            self.claimable_amount(to, token_id, nonce)
                .update(|amt| *amt += amount);
        } else {
            self.send().direct(to, token_id, nonce, amount, data);
        }
    }

    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }

    fn try_set_bid_cut_percentage(&self, new_cut_percentage: u64) {
        require!(
            new_cut_percentage > 0 && new_cut_percentage < PERCENTAGE_TOTAL,
            "Invalid percentage value, should be between 0 and 10,000"
        );

        self.bid_cut_percentage()
            .set(&BigUint::from(new_cut_percentage));
    }
}
