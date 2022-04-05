#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

pub mod auction;
use auction::*;
pub mod offer;
use offer::*;

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
        #[var_args] opt_min_bid_diff: OptionalValue<BigUint>,
        #[var_args] opt_sft_max_one_per_payment: OptionalValue<bool>,
        #[var_args] opt_accepted_payment_token_nonce: OptionalValue<u64>,
        #[var_args] opt_start_time: OptionalValue<u64>,
    ) -> u64 {
        require!(nft_amount >= NFT_AMOUNT, "Must tranfer at least one");

        require!(
            self.auctions_by_token(&nft_type, nft_nonce).is_empty(),
            "An auction for this token already exists"
        );

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
            auctioned_token: EsdtToken {
                token_type: nft_type.clone(),
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
        self.auctions_by_token(&nft_type, nft_nonce)
            .insert(auction_id);
        self.auctions_by_address(&auction.original_owner)
            .insert(auction_id);

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

        if auction.current_bid > 0 {
            if let Some(max_bid) = &auction.max_bid {
                if &payment_amount < max_bid {
                    require!(
                        (&payment_amount - &auction.current_bid) >= auction.min_bid_diff,
                        "The difference from the last bid must be higher"
                    );
                }
            }
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
        #[var_args] opt_sft_buy_amount: OptionalValue<BigUint>,
    ) {
        let mut auction = self.try_get_auction(auction_id);
        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();

        let sft_buy_amount = match opt_sft_buy_amount {
            OptionalValue::Some(amt) => amt,
            OptionalValue::None => BigUint::from(NFT_AMOUNT),
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
    fn withdraw_auction(&self, auction_id: u64) {
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

        let nft_type = &auction.auctioned_token.token_type;
        let nft_nonce = auction.auctioned_token.nonce;
        let nft_amount = &auction.nr_auctioned_tokens;

        self.auction_by_id(auction_id).clear();
        self.auctions_by_token(&nft_type, nft_nonce)
            .remove(&auction_id);
        self.auctions_by_address(&caller).remove(&auction_id);
        self.transfer_or_save_payment(&caller, nft_type, nft_nonce, nft_amount, b"returned token");

        self.emit_withdraw_event(auction_id, auction);
    }

    #[endpoint(claimTokens)]
    fn claim_tokens(
        &self,
        claim_destination: ManagedAddress,
        #[var_args] token_nonce_pairs: MultiValueEncoded<MultiValue2<TokenIdentifier, u64>>,
    ) -> MultiValue2<BigUint, ManagedVec<EsdtTokenPayment<Self::Api>>> {
        let caller = self.blockchain().get_caller();
        let mut egld_payment_amount = BigUint::zero();
        let mut output_payments = ManagedVec::new();

        for pair in token_nonce_pairs {
            let (token_id, token_nonce) = pair.into_tuple();
            let amount_mapper = self.claimable_amount(&caller, &token_id, token_nonce);
            let amount = amount_mapper.get();

            if amount > 0 {
                amount_mapper.clear();

                if token_id.is_egld() {
                    egld_payment_amount = amount;
                } else {
                    output_payments.push(EsdtTokenPayment::new(token_id, token_nonce, amount));
                }
            }
        }

        if egld_payment_amount > 0 {
            self.send()
                .direct_egld(&claim_destination, &egld_payment_amount, &[]);
        }
        if !output_payments.is_empty() {
            self.send()
                .direct_multi(&claim_destination, &output_payments, &[]);
        }

        (egld_payment_amount, output_payments).into()
    }

    #[payable("*")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        nft_amount: BigUint,
        deadline: u64,
    ) -> u64 {
        require!(nft_nonce > 0, "Can't place offers for fungible tokens");
        require!(
            nft_amount == BigUint::from(NFT_AMOUNT),
            "The quantity must be equal to 1!"
        );

        let current_time = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();

        if !payment_token.is_egld() {
            require!(
                payment_token.is_valid_esdt_identifier(),
                "The payment token is not valid!"
            );
        }
        require!(
            nft_type.is_valid_esdt_identifier(),
            "The NFT token is not valid!"
        );
        require!(
            !self
                .offer_exists(&caller, &nft_type, nft_nonce, &payment_token)
                .get(),
            "Offer already exists!"
        );

        require!(deadline > current_time, "Deadline can't be in the past!");

        let marketplace_cut_percentage = self.bid_cut_percentage().get();
        let creator_royalties_percentage = self.get_nft_info(&nft_type, nft_nonce).royalties;

        let offer_id = self.last_valid_offer_id().get() + 1;
        self.last_valid_offer_id().set(&offer_id);

        let offer = Offer {
            offer_token: EsdtToken {
                token_type: nft_type,
                nonce: nft_nonce,
            },
            payment_token: EsdtToken {
                token_type: payment_token,
                nonce: payment_token_nonce,
            },
            quantity: nft_amount,
            offer_price: payment_amount,
            start_time: current_time,
            deadline,
            offer_owner: caller,
            marketplace_cut_percentage,
            creator_royalties_percentage,
        };

        self.offer_by_id(offer_id).set(&offer);
        self.offers_by_address(&offer.offer_owner).insert(offer_id);
        self.offers_by_token(&offer.offer_token.token_type, nft_nonce)
            .insert(offer_id);

        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    #[endpoint(withdrawOffer)]
    fn withdraw_offer(&self, offer_id: u64) {
        let offer = self.try_get_offer(offer_id);
        let caller = self.blockchain().get_caller();

        require!(
            offer.offer_owner == caller,
            "Only the address that placed the offer can withdraw it!"
        );

        self.send().direct(
            &caller,
            &offer.payment_token.token_type,
            offer.payment_token.nonce,
            &offer.offer_price,
            self.get_transfer_data(&caller, b"Withdraw offer!"),
        );

        self.offer_exists(
            &offer.offer_owner,
            &offer.offer_token.token_type,
            offer.offer_token.nonce,
            &offer.payment_token.token_type,
        )
        .clear();
        self.offers_by_token(&offer.offer_token.token_type, offer.offer_token.nonce)
            .remove(&offer_id);
        self.offers_by_address(&offer.offer_owner).remove(&offer_id);
        self.offer_by_id(offer_id).clear();

        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(
        &self,
        #[payment_token] nft_token: TokenIdentifier,
        #[payment_nonce] nft_token_nonce: u64,
        #[payment_amount] nft_amount: BigUint,
        offer_id: u64,
    ) {
        let offer = self.try_get_offer(offer_id);
        let seller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();
        require!(current_time <= offer.deadline, "Offer has expired!");
        require!(offer.offer_owner != seller, "Cannot accept your own offer!");

        require!(
            nft_token == offer.offer_token.token_type,
            "The sent token type is different from the offer!"
        );
        require!(
            nft_token_nonce == offer.offer_token.nonce,
            "The sent token nonce is different from the offer!"
        );
        require!(
            nft_amount == offer.quantity,
            "The number of tokens sent is different from the offer!"
        );

        let has_no_active_auctions = self
            .auctions_by_token(&nft_token, nft_token_nonce)
            .is_empty();

        // if an NFT is bought, all the active auctions will be canceled
        if !has_no_active_auctions {
            self.clear_auctions_for_token(&nft_token, nft_token_nonce);
        };

        let token_info = self.get_nft_info(&offer.offer_token.token_type, offer.offer_token.nonce);
        let creator_royalties_percentage = token_info.royalties;
        require!(
            &offer.marketplace_cut_percentage + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );

        self.transfer_or_save_payment(
            &offer.offer_owner,
            &offer.offer_token.token_type,
            offer.offer_token.nonce,
            &offer.quantity,
            b"Token bought!",
        );

        let offer_split_amounts = self.calculate_accepted_offer_split(&offer);
        let marketplace_owner = self.blockchain().get_owner_address();

        // NFT marketplace revenue
        self.transfer_or_save_payment(
            &marketplace_owner,
            &offer.payment_token.token_type,
            offer.payment_token.nonce,
            &offer_split_amounts.marketplace,
            b"Marketplace sale fees!",
        );

        // NFT creator revenue
        self.transfer_or_save_payment(
            &token_info.creator,
            &offer.payment_token.token_type,
            offer.payment_token.nonce,
            &offer_split_amounts.creator,
            b"Creator royalties!",
        );

        // NFT seller revenue
        self.transfer_or_save_payment(
            &seller,
            &offer.payment_token.token_type,
            offer.payment_token.nonce,
            &offer_split_amounts.seller,
            b"Token sold!",
        );

        self.offers_by_token(&offer.payment_token.token_type, offer.payment_token.nonce)
            .remove(&offer_id);
        self.offers_by_address(&offer.offer_owner).remove(&offer_id);
        self.offer_by_id(offer_id).clear();

        self.emit_accept_offer_event(offer_id, offer, &seller);
    }

    // private

    fn try_get_auction(&self, auction_id: u64) -> Auction<Self::Api> {
        require!(
            self.does_auction_exist(auction_id),
            "Auction does not exist"
        );
        self.auction_by_id(auction_id).get()
    }

    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        require!(self.does_offer_exist(offer_id), "Offer does not exist!");
        self.offer_by_id(offer_id).get()
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

    fn calculate_accepted_offer_split(
        &self,
        offer: &Offer<Self::Api>,
    ) -> BidSplitAmounts<Self::Api> {
        let creator_royalties =
            self.calculate_cut_amount(&offer.offer_price, &offer.creator_royalties_percentage);
        let offer_cut_amount =
            self.calculate_cut_amount(&offer.offer_price, &offer.marketplace_cut_percentage);
        let mut seller_amount_to_send = offer.offer_price.clone();
        seller_amount_to_send -= &creator_royalties;
        seller_amount_to_send -= &offer_cut_amount;

        BidSplitAmounts {
            creator: creator_royalties,
            marketplace: offer_cut_amount,
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

    fn get_transfer_data(&self, address: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(address) {
            &[]
        } else {
            data
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

    fn clear_auctions_for_token(&self, nft_token: &TokenIdentifier, nft_token_nonce: u64) {
        for auction_id in self.auctions_by_token(&nft_token, nft_token_nonce).iter() {
            let auction = self.try_get_auction(auction_id);
            require!(
                &auction.auctioned_token.token_type == nft_token,
                "The auctioned token does not match the offer!"
            );
            require!(
                auction.auctioned_token.nonce == nft_token_nonce,
                "The auctioned token nonce does not match the offer!"
            );
            require!(
                auction.nr_auctioned_tokens == BigUint::from(NFT_AMOUNT),
                "The token amount for sale is higher than 1!"
            );

            self.withdraw_auction(auction_id);
        }
    }
}
