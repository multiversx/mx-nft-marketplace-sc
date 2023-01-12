////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_nft_marketplace
    (
        acceptOffer
        addTokensToWhitelist
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
        getWhitelistedTokens
        isPaused
        pause
        removeTokensFromWhitelist
        sendOffer
        setCutPercentage
        unpause
        withdraw
        withdrawAuctionAndAcceptOffer
        withdrawOffer
    )
}

elrond_wasm_node::wasm_empty_callback! {}
