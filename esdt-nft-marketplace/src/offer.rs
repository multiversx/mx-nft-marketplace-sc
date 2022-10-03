elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::auction::NFT_AMOUNT;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Offer<M: ManagedTypeApi> {
    pub offer_token: EsdtTokenPayment<M>,
    pub payment: EgldOrEsdtTokenPayment<M>,
    pub start_time: u64,
    pub deadline: u64,
    pub offer_owner: ManagedAddress<M>,
}

#[elrond_wasm::module]
pub trait OfferModule:
    crate::auction::AuctionModule
    + crate::token_distribution::TokenDistributionModule
    + crate::events::EventsModule
    + crate::common_util_functions::CommonUtilFunctions
    + elrond_wasm_modules::pause::PauseModule
{
    #[payable("*")]
    #[endpoint(sendOffer)]
    fn send_offer(
        &self,
        desired_nft_id: TokenIdentifier,
        desired_nft_nonce: u64,
        deadline: u64,
        opt_auction_id: OptionalValue<u64>,
    ) -> u64 {
        self.require_not_paused();
        require!(
            desired_nft_nonce > 0,
            "Can't place offers for fungible tokens"
        );

        let payment = self.call_value().egld_or_single_esdt();
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            payment.amount > 0u64,
            "Payment amount must be greater than 0"
        );
        require!(deadline > current_time, "Deadline can't be in the past!");

        let offer_token = EsdtTokenPayment::new(
            desired_nft_id.clone(),
            desired_nft_nonce,
            BigUint::from(NFT_AMOUNT),
        );

        let offer = Offer {
            offer_token,
            payment,
            start_time: current_time,
            deadline,
            offer_owner: caller,
        };

        let token_amount_in_marketplace = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(desired_nft_id.clone()),
            desired_nft_nonce,
        );
        if token_amount_in_marketplace > 0 {
            match opt_auction_id {
                OptionalValue::Some(auction_id) => {
                    let auction = self.try_get_auction(auction_id);
                    require!(
                        auction.auctioned_tokens.token_identifier == desired_nft_id,
                        "The auction does not contain the NFT"
                    );
                    require!(
                        auction.current_bid == BigUint::zero(),
                        "NFT auction has active bids"
                    );
                }
                OptionalValue::None => sc_panic!("Must provide the auction id"),
            };
        }

        let offer_id = self.last_valid_offer_id().get() + 1;
        self.last_valid_offer_id().set(&offer_id);
        self.offer_by_id(offer_id).set(&offer);
        self.offers_by_address(&offer.offer_owner).insert(offer_id);
        self.offers_by_token(
            &offer.offer_token.token_identifier,
            offer.offer_token.token_nonce,
        )
        .insert(offer_id);

        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    #[endpoint(withdrawOffer)]
    fn withdraw_offer(&self, offer_id: u64) {
        self.require_not_paused();
        let offer = self.try_get_offer(offer_id);
        let caller = self.blockchain().get_caller();

        require!(
            offer.offer_owner == caller,
            "Only the address that placed the offer can withdraw it!"
        );

        self.send().direct(
            &caller,
            &offer.payment.token_identifier,
            offer.payment.token_nonce,
            &offer.payment.amount,
        );

        self.offers_by_token(
            &offer.offer_token.token_identifier,
            offer.offer_token.token_nonce,
        )
        .swap_remove(&offer_id);
        self.offers_by_address(&offer.offer_owner)
            .swap_remove(&offer_id);
        self.offer_by_id(offer_id).clear();

        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(&self, offer_id: u64) {
        self.require_not_paused();
        let offer_nft = self.call_value().single_esdt();
        let offer = self.try_get_offer(offer_id);
        require!(offer_nft.amount == NFT_AMOUNT, "You can only send NFTs");
        require!(
            offer_nft.token_identifier == offer.offer_token.token_identifier,
            "The sent token type is different from the offer"
        );
        require!(
            offer_nft.token_nonce == offer.offer_token.token_nonce,
            "The sent token nonce is different from the offer"
        );
        self.accept_offer_common(offer_id, offer);
    }

    #[endpoint(withdrawAuctionAndAcceptOffer)]
    fn withdraw_auction_and_accept_offer(&self, auction_id: u64, offer_id: u64) {
        self.require_not_paused();
        let auction = self.try_get_auction(auction_id);
        let offer = self.try_get_offer(offer_id);
        require!(
            auction.auctioned_tokens.token_identifier == offer.offer_token.token_identifier,
            "The token id from the auction does not match the one from the offer"
        );
        require!(
            auction.current_bid == BigUint::zero(),
            "NFT auction has active bids"
        );
        let token_amount_in_marketplace = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(offer.offer_token.token_identifier.clone()),
            offer.offer_token.token_nonce,
        );
        require!(
            token_amount_in_marketplace > 0u64,
            "The NFT must be in the contract to accept the offer"
        );

        self.withdraw_auction_common(auction_id, auction);
        self.accept_offer_common(offer_id, offer);
    }

    fn accept_offer_common(&self, offer_id: u64, offer: Offer<Self::Api>) {
        let seller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();
        require!(current_time < offer.deadline, "Offer has expired");
        require!(offer.offer_owner != seller, "Cannot accept your own offer");

        let marketplace_cut_percentage = self.bid_cut_percentage().get();
        self.distribute_tokens_after_offer_accept(&offer, &seller, &marketplace_cut_percentage);
        self.offers_by_token(
            &offer.offer_token.token_identifier,
            offer.offer_token.token_nonce,
        )
        .swap_remove(&offer_id);
        self.offers_by_address(&offer.offer_owner)
            .swap_remove(&offer_id);
        self.offer_by_id(offer_id).clear();

        self.emit_accept_offer_event(offer_id, offer, &seller);
    }

    fn get_transfer_data(&self, address: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(address) {
            &[]
        } else {
            data
        }
    }

    #[view(getFullOfferData)]
    fn try_get_offer(&self, offer_id: u64) -> Offer<Self::Api> {
        let offer_mapper = self.offer_by_id(offer_id);
        require!(!offer_mapper.is_empty(), "Offer does not exist");
        offer_mapper.get()
    }

    #[view(getLastValidOfferId)]
    #[storage_mapper("lastValidOfferId")]
    fn last_valid_offer_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("offerById")]
    fn offer_by_id(&self, offer_id: u64) -> SingleValueMapper<Offer<Self::Api>>;

    #[view(getOffersByAddress)]
    #[storage_mapper("offersByAddress")]
    fn offers_by_address(&self, address: &ManagedAddress) -> UnorderedSetMapper<u64>;

    #[view(getOffersByToken)]
    #[storage_mapper("offersByToken")]
    fn offers_by_token(
        &self,
        token_id: &TokenIdentifier,
        token_nonce: u64,
    ) -> UnorderedSetMapper<u64>;
}
