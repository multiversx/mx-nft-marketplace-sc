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
        endAuction
        getClaimableAmount
        getFullAuctionData
        getFullOfferData
        getLastValidAuctionId
        getLastValidOfferId
        getMarketplaceCutPercentage
        getOffersByAddress
        getOffersByToken
        isPaused
        pause
        sendOffer
        setCutPercentage
        unpause
        withdraw
        withdrawAuctionAndAcceptOffer
        withdrawOffer
    )
}

elrond_wasm_node::wasm_empty_callback! {}
