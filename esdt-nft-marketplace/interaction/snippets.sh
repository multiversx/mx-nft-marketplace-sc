ALICE="/home/elrond/Downloads/devnetWalletKey.pem" # PEM path
ADDRESS=$(erdpy data load --key=address-devnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D

deploy() {
    local MARKETPLACE_BID_CUT_PERCENTAGE=1000 # 10%

    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --arguments ${MARKETPLACE_BID_CUT_PERCENTAGE} \
    --send --outfile="deploy-devnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-devnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

auctionToken() {
    local MIN_BID=1
    local MAX_BID=1000000000000000000 # 1 EGLD
    local DEADLINE=2000000000
    local PAYMENT_TOKEN=0x45474c44 # "EGLD"

    local NFT_TOKEN_ID=0x544553544e46542d343535373264 # TESTNFT-45572d
    local NFT_NONCE=1
    local NFT_QUANTITY=1
    local MARKET_SC_ADDRESS=0x00000000000000000500c539aa50b6a586b1632e4d59aec15720acb3244479d5
    local FUNC_NAME=0x61756374696f6e546f6b656e # "auctionToken"

    erdpy --verbose contract call erd1yyfyrzu7wu5lh8jqegtj5klc0y6z8n8tjzlsw2zd00tu9pwl082sfv6x8c --recall-nonce --pem=${ALICE} \
    --gas-limit=50000000 --function="ESDTNFTTransfer" \
    --arguments ${NFT_TOKEN_ID} ${NFT_NONCE} ${NFT_QUANTITY} ${MARKET_SC_ADDRESS} ${FUNC_NAME} ${MIN_BID} ${MAX_BID} ${DEADLINE} ${PAYMENT_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

endAuction() {
    local AUCTION_ID=1

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=50000000 --function="endAuction" \
    --arguments ${AUCTION_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}