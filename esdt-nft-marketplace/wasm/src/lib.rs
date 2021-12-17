////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        init
        auctionToken
        bid
        buySft
        claimTokens
        doesAuctionExist
        endAuction
        getAuctionType
        getAuctionedToken
        getClaimableAmount
        getCurrentWinner
        getCurrentWinningBid
        getDeadline
        getFullAuctionData
        getLastValidAuctionId
        getMarketplaceCutPercentage
        getMinMaxBid
        getOriginalOwner
        getPaymentTokenForAuction
        getStartTime
        setCutPercentage
        withdraw
    )
}

elrond_wasm_node::wasm_empty_callback! {}
