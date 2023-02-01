LEDGER_INDEX=0
ROOT=".."

CONTRACT_ADDRESS=$(mxpy data load --key=address-nft-marketplace)
DEPLOY_TRANSACTION=$(mxpy data load --key=deploy-tx-nft-marketplace)

PROXY=https://testnet-gateway.multiversx.com
CHAIN_ID=T

GAS=100000000
BID_PERCENT=1000


deploy() {
  mxpy contract deploy --bytecode=${ROOT}/output/esdt-nft-marketplace.wasm \
  --ledger --ledger-account-index=$LEDGER_INDEX \
  --proxy=${PROXY} --chain=${CHAIN_ID} --gas-limit=${GAS} \
  --arguments ${BID_PERCENT} \
  --outfile="deploy.json" --recall-nonce --send  || return

  TX=$(mxpy data parse --file="deploy.json" --expression="data['emittedTransactionHash']")
  ADDRESS=$(mxpy data parse --file="deploy.json" --expression="data['contractAddress']")

  mxpy data store --key=address-nft-marketplace --value=${ADDRESS}
  mxpy data store --key=deploy-tx-nft-marketplace --value=${TX}

  echo ""
  echo "Smart contract address: ${ADDRESS}"
}

pause() {
  function_name="pause"
  mxpy contract call $CONTRACT_ADDRESS \
  --ledger --ledger-account-index=$LEDGER_INDEX \
  --proxy=${PROXY} --chain=${CHAIN_ID} --gas-limit=5000000 \
  --function ${function_name} \
  --recall-nonce --send  || return
}

unpause() {
  function_name="unpause"
  mxpy contract call $CONTRACT_ADDRESS \
  --ledger --ledger-account-index=$LEDGER_INDEX \
  --proxy=${PROXY} --chain=${CHAIN_ID} --gas-limit=5000000 \
  --function ${function_name} \
  --recall-nonce --send  || return
}

upgrade() {
  mxpy contract upgrade $CONTRACT_ADDRESS --bytecode=${ROOT}/output/esdt-nft-marketplace.wasm \
  --ledger --ledger-account-index=$LEDGER_INDEX \
  --proxy=${PROXY} --chain=${CHAIN_ID} --gas-limit=${GAS} \
  --arguments ${BID_PERCENT} \
  --outfile="upgrade.json" --recall-nonce --send  || return
}
