elrond_wasm::imports!();
elrond_wasm::derive_imports!();

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
        desired_amount: BigUint,
        deadline: u64,
        opt_auction_id: OptionalValue<u64>,
    ) -> u64 {
        self.require_not_paused();
        require!(
            desired_nft_nonce > 0,
            "Can't place offers for fungible tokens"
        );
        require!(desired_amount > 0, "Amount must be greater than 0");

        let payment = self.call_value().egld_or_single_esdt();
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            payment.amount > 0u64,
            "Payment amount must be greater than 0"
        );
        require!(deadline > current_time, "Deadline can't be in the past!");

        self.check_nft_in_marketplace(
            &desired_nft_id,
            desired_nft_nonce,
            &desired_amount,
            opt_auction_id,
        );

        let offer_token = EsdtTokenPayment::new(desired_nft_id, desired_nft_nonce, desired_amount);

        let offer = Offer {
            offer_token,
            payment,
            start_time: current_time,
            deadline,
            offer_owner: caller,
        };

        let offer_id = self.last_valid_offer_id().get() + 1;
        self.last_valid_offer_id().set(offer_id);
        self.offer_by_id(offer_id).set(&offer);

        self.emit_offer_token_event(offer_id, offer);

        offer_id
    }

    fn check_nft_in_marketplace(
        &self,
        desired_nft_id: &TokenIdentifier,
        desired_nft_nonce: u64,
        desired_amount: &BigUint,
        opt_auction_id: OptionalValue<u64>,
    ) {
        let token_amount_in_marketplace = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(desired_nft_id.clone()),
            desired_nft_nonce,
        );
        if &token_amount_in_marketplace >= desired_amount {
            match opt_auction_id {
                OptionalValue::Some(auction_id) => {
                    let auction = self.try_get_auction(auction_id);
                    require!(
                        &auction.auctioned_tokens.token_identifier == desired_nft_id,
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

        self.offer_by_id(offer_id).clear();
        self.emit_withdraw_offer_event(offer_id, offer);
    }

    #[payable("*")]
    #[endpoint(acceptOffer)]
    fn accept_offer(&self, offer_id: u64) {
        self.require_not_paused();
        let caller = self.blockchain().get_caller();
        let offer_nft = self.call_value().single_esdt();
        let offer = self.try_get_offer(offer_id);
        require!(
            offer_nft.amount == offer.offer_token.amount,
            "The token amount is different from the offer"
        );
        require!(
            offer_nft.token_identifier == offer.offer_token.token_identifier,
            "The sent token type is different from the offer"
        );
        require!(
            offer_nft.token_nonce == offer.offer_token.token_nonce,
            "The sent token nonce is different from the offer"
        );
        self.accept_offer_common(&caller, offer_id, offer);
    }

    #[endpoint(withdrawAuctionAndAcceptOffer)]
    fn withdraw_auction_and_accept_offer(&self, auction_id: u64, offer_id: u64) {
        self.require_not_paused();
        let caller = self.blockchain().get_caller();
        let auction = self.try_get_auction(auction_id);
        let offer = self.try_get_offer(offer_id);
        require!(
            auction.auctioned_tokens.token_identifier == offer.offer_token.token_identifier,
            "The token id from the auction does not match the one from the offer"
        );
        require!(
            auction.auctioned_tokens.amount == offer.offer_token.amount,
            "The amount from the auction does not match the one from the offer"
        );
        require!(
            auction.current_bid == BigUint::zero(),
            "NFT auction has active bids"
        );

        self.withdraw_auction_common(&caller, auction_id, auction);
        self.accept_offer_common(&caller, offer_id, offer);
    }

    fn accept_offer_common(&self, seller: &ManagedAddress, offer_id: u64, offer: Offer<Self::Api>) {
        let current_time = self.blockchain().get_block_timestamp();
        require!(current_time < offer.deadline, "Offer has expired");
        require!(&offer.offer_owner != seller, "Cannot accept your own offer");

        let marketplace_cut_percentage = self.bid_cut_percentage().get();
        self.distribute_tokens_after_offer_accept(&offer, seller, &marketplace_cut_percentage);
        self.offer_by_id(offer_id).clear();

        self.emit_accept_offer_event(offer_id, offer, seller);
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
}
