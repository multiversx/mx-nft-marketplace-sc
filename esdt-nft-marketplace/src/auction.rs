elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// temporary until EsdtTokenType is removed from EsdtTokenPayment's encoding
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct CustomEsdtTokenPayment<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
    pub amount: BigUint<M>,
}

impl<M: ManagedTypeApi> CustomEsdtTokenPayment<M> {
    pub fn new(token_identifier: TokenIdentifier<M>, token_nonce: u64, amount: BigUint<M>) -> Self {
        CustomEsdtTokenPayment {
            token_identifier,
            token_nonce,
            amount,
        }
    }
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Auction<M: ManagedTypeApi> {
    pub auctioned_tokens: CustomEsdtTokenPayment<M>,
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

pub struct BidSplitAmounts<M: ManagedTypeApi> {
    pub creator: BigUint<M>,
    pub marketplace: BigUint<M>,
    pub seller: BigUint<M>,
}
