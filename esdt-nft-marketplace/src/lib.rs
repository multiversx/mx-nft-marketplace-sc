#![no_std]

elrond_wasm::imports!();

pub mod marketplace_main;
pub mod marketplace_offer;
pub mod auction;
pub mod offer;
pub mod storage;
pub mod views;
pub mod events;

#[elrond_wasm::contract]
pub trait EsdtNftMarketplace:
    marketplace_main::MarketplaceAuctionModule
    + marketplace_offer::MarketplaceOfferModule
    + storage::StorageModule
    + events::EventsModule
    + views::ViewsModule
{
    #[init]
    fn init(&self, bid_cut_percentage: u64) {
        self.try_set_bid_cut_percentage(bid_cut_percentage);
    }
}


