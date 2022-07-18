elrond_wasm::imports!();

use crate::auction::{Auction, AuctionType, NFT_AMOUNT, PERCENTAGE_TOTAL};

pub struct BidSplitAmounts<M: ManagedTypeApi> {
    pub creator: BigUint<M>,
    pub marketplace: BigUint<M>,
    pub seller: BigUint<M>,
}

#[elrond_wasm::module]
pub trait TokenDistributionModule:
    crate::common_util_functions::CommonUtilFunctions + elrond_wasm_modules::pause::PauseModule
{
    #[endpoint(claimTokens)]
    fn claim_tokens(
        &self,
        claim_destination: ManagedAddress,
        token_nonce_pairs: MultiValueEncoded<MultiValue2<EgldOrEsdtTokenIdentifier, u64>>,
    ) -> MultiValue2<BigUint, ManagedVec<EsdtTokenPayment<Self::Api>>> {
        self.require_not_paused();

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
                    output_payments.push(EsdtTokenPayment::new(
                        token_id.unwrap_esdt(),
                        token_nonce,
                        amount,
                    ));
                }
            }
        }

        if egld_payment_amount > 0 {
            self.send()
                .direct_egld(&claim_destination, &egld_payment_amount);
        }
        if !output_payments.is_empty() {
            self.send()
                .direct_multi(&claim_destination, &output_payments);
        }

        (egld_payment_amount, output_payments).into()
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
        let nft_type = &auction.auctioned_tokens.token_identifier;
        let nft_nonce = auction.auctioned_tokens.token_nonce;

        if !auction.current_winner.is_zero() {
            let nft_info = self.get_nft_info(nft_type, nft_nonce);
            let token_id = &auction.payment_token;
            let nonce = auction.payment_nonce;
            let bid_split_amounts = self.calculate_winning_bid_split(auction);

            // send part as cut for contract owner
            let owner = self.blockchain().get_owner_address();
            self.transfer_or_save_payment(&owner, token_id, nonce, &bid_split_amounts.marketplace);

            // send part as royalties to creator
            self.transfer_or_save_payment(
                &nft_info.creator,
                token_id,
                nonce,
                &bid_split_amounts.creator,
            );

            // send rest of the bid to original owner
            self.transfer_or_save_payment(
                &auction.original_owner,
                token_id,
                nonce,
                &bid_split_amounts.seller,
            );

            // send NFT to auction winner
            let nft_amount = BigUint::from(NFT_AMOUNT);
            let nft_amount_to_send = match auction.auction_type {
                AuctionType::Nft => &nft_amount,
                AuctionType::SftOnePerPayment => match opt_sft_amount {
                    Some(amt) => amt,
                    None => &nft_amount,
                },
                _ => &auction.auctioned_tokens.amount,
            };
            self.transfer_or_save_payment(
                &auction.current_winner,
                &EgldOrEsdtTokenIdentifier::esdt(nft_type.clone()),
                nft_nonce,
                nft_amount_to_send,
            );
        } else {
            // return to original owner
            self.transfer_or_save_payment(
                &auction.original_owner,
                &EgldOrEsdtTokenIdentifier::esdt(nft_type.clone()),
                nft_nonce,
                &auction.auctioned_tokens.amount,
            );
        }
    }

    fn transfer_or_save_payment(
        &self,
        to: &ManagedAddress,
        token_id: &EgldOrEsdtTokenIdentifier,
        nonce: u64,
        amount: &BigUint,
    ) {
        if amount == &0 {
            return;
        }

        if self.blockchain().is_smart_contract(to) {
            self.claimable_amount(to, token_id, nonce)
                .update(|amt| *amt += amount);
        } else {
            self.send().direct(to, token_id, nonce, amount);
        }
    }

    #[view(getClaimableAmount)]
    #[storage_mapper("claimableAmount")]
    fn claimable_amount(
        &self,
        address: &ManagedAddress,
        token_id: &EgldOrEsdtTokenIdentifier,
        token_nonce: u64,
    ) -> SingleValueMapper<BigUint>;
}
