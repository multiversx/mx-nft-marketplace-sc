////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        acceptOffer
        auctionToken
        bid
        buySft
        claimTokens
        doesAuctionExist
        doesOfferExist
        endAuction
        getAuctionType
        getAuctionedToken
        getAuctionsByToken
        getClaimableAmount
        getCurrentWinner
        getCurrentWinningBid
        getDeadline
        getFullAuctionData
        getLastValidAuctionId
        getLastValidOfferId
        getMarketplaceCutPercentage
        getMinMaxBid
        getOfferExists
        getOffersByAddress
        getOffersByToken
        getOriginalOwner
        getPaymentTokenForAuction
        getStartTime
        sendOffer
        setCutPercentage
        withdraw
        withdrawOffer
    )
}

elrond_wasm_node::wasm_empty_callback! {}
