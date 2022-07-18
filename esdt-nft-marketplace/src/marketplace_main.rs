#![no_std]

elrond_wasm::imports!();

use crate::auction::PERCENTAGE_TOTAL;

pub mod auction;
pub mod bidding;
pub mod common_util_functions;
pub mod events;
pub mod token_distribution;

#[elrond_wasm::contract]
pub trait EsdtNftMarketplace:
    auction::AuctionModule
    + bidding::BiddingModule
    + token_distribution::TokenDistributionModule
    + events::EventsModule
    + common_util_functions::CommonUtilFunctions
    + elrond_wasm_modules::pause::PauseModule
{
    #[init]
    fn init(&self, bid_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
    }

    #[only_owner]
    #[endpoint(setCutPercentage)]
    fn set_percentage_cut(&self, new_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(new_cut_percentage);
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
