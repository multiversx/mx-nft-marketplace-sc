elrond_wasm::imports!();

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%

use crate::{
    auction::{BidSplitAmounts, EsdtToken},
    events, marketplace_main,
    offer::Offer,
    storage, views,
};

#[elrond_wasm::module]
pub trait MarketplaceOfferModule:
    storage::StorageModule
    + views::ViewsModule
    + events::EventsModule
    + marketplace_main::MarketplaceAuctionModule
{
    #[payable("*")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        deadline: u64,
    ) -> u64 {
        require!(nft_nonce > 0, "Can't place offers for fungible tokens");

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
        //TODO - tests fail
        // let creator_royalties_percentage = self.get_nft_info(&nft_type, nft_nonce).royalties;

        //We can use this to check if an exact offer like this exists
        //It can be modified to check only if the caller has any active offers and so on
        self.offer_exists(
            &caller,
            &nft_type,
            nft_nonce,
            &payment_token,
        )
        .set(&true);

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
            offer_price: payment_amount,
            start_time: current_time,
            deadline,
            offer_owner: caller,
            marketplace_cut_percentage,
            creator_royalties_percentage: BigUint::zero(),
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
            .swap_remove(&offer_id);
        self.offers_by_address(&offer.offer_owner)
            .swap_remove(&offer_id);
        self.offer_by_id(offer_id).clear();

        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(
        &self,
        #[payment_token] nft_token: TokenIdentifier,
        #[payment_nonce] nft_token_nonce: u64,
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

        let has_no_active_auctions = self
            .auctions_by_token(&nft_token, nft_token_nonce)
            .is_empty();

        // V1 - if an NFT is bought, all the active auctions will be canceled
        // if !has_no_active_auctions {
        //     self.clear_auctions_for_token(&nft_token, nft_token_nonce);
        // };

        // V2 - the NFT must not have active auctions
        require!(has_no_active_auctions, "The NFT has active auctions!");

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
            &BigUint::from(1u64),
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

        self.offer_exists(
            &offer.offer_owner,
            &offer.offer_token.token_type,
            offer.offer_token.nonce,
            &offer.payment_token.token_type,
        )
        .clear();
        self.offers_by_token(&offer.offer_token.token_type, offer.offer_token.nonce)
            .swap_remove(&offer_id);
        self.offers_by_address(&offer.offer_owner)
            .swap_remove(&offer_id);
        self.offer_by_id(offer_id).clear();

        self.emit_accept_offer_event(offer_id, offer, &seller);
    }

    // private

    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        require!(self.does_offer_exist(offer_id), "Offer does not exist!");
        self.offer_by_id(offer_id).get()
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

    fn get_transfer_data(&self, address: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(address) {
            &[]
        } else {
            data
        }
    }

    // fn clear_auctions_for_token(&self, nft_token: &TokenIdentifier, nft_token_nonce: u64) {
    //     for auction_id in self.auctions_by_token(&nft_token, nft_token_nonce).iter() {
    //         let auction = self.try_get_auction(auction_id);
    //         require!(
    //             &auction.auctioned_token.token_type == nft_token,
    //             "The auctioned token does not match the offer!"
    //         );
    //         require!(
    //             auction.auctioned_token.nonce == nft_token_nonce,
    //             "The auctioned token nonce does not match the offer!"
    //         );
    //         require!(
    //             auction.nr_auctioned_tokens == BigUint::from(NFT_AMOUNT),
    //             "The token amount for sale is higher than 1!"
    //         );

    //         self.withdraw_auction(auction_id);
    //     }
    // }
}
