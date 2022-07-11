elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::NestedDecodeInput;

#[derive(TopEncode, TypeAbi)]
pub struct Auction<M: ManagedTypeApi> {
    pub auctioned_tokens: EsdtTokenPayment<M>,
    pub auction_type: AuctionType,

    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub min_bid: BigUint<M>,
    pub max_bid: Option<BigUint<M>>,
    pub start_time: u64,
    pub deadline: u64,

    pub original_owner: ManagedAddress<M>,
    pub current_bid: BigUint<M>,
    pub current_winner: ManagedAddress<M>,
    pub marketplace_cut_percentage: BigUint<M>,
    pub creator_royalties_percentage: BigUint<M>,

    // Add new fields here for backwards compatibility
    pub min_bid_diff: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum AuctionType {
    None,
    Nft,
    SftAll,
    SftOnePerPayment,
}

impl<M: ManagedTypeApi> TopDecode for Auction<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: elrond_codec::TopDecodeInput,
    {
        let mut input = input.into_nested_buffer();
        let auctioned_tokens = EsdtTokenPayment::dep_decode(&mut input)?;
        let auction_type = AuctionType::dep_decode(&mut input)?;
        let payment_token = EgldOrEsdtTokenIdentifier::dep_decode(&mut input)?;
        let payment_nonce = u64::dep_decode(&mut input)?;
        let min_bid = BigUint::dep_decode(&mut input)?;
        let max_bid = Option::<BigUint<M>>::dep_decode(&mut input)?;
        let start_time = u64::dep_decode(&mut input)?;
        let deadline = u64::dep_decode(&mut input)?;
        let original_owner = ManagedAddress::dep_decode(&mut input)?;
        let current_bid = BigUint::dep_decode(&mut input)?;
        let current_winner = ManagedAddress::dep_decode(&mut input)?;
        let marketplace_cut_percentage = BigUint::dep_decode(&mut input)?;
        let creator_royalties_percentage = BigUint::dep_decode(&mut input)?;

        let min_bid_diff = if input.is_depleted() {
            BigUint::zero()
        } else {
            BigUint::dep_decode(&mut input)?
        };

        Result::Ok(Auction {
            auctioned_tokens,
            auction_type,
            payment_token,
            payment_nonce,
            min_bid,
            max_bid,
            start_time,
            deadline,
            original_owner,
            current_bid,
            current_winner,
            marketplace_cut_percentage,
            creator_royalties_percentage,
            min_bid_diff,
        })
    }
}
