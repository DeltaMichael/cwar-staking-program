#!/bin/bash
set -e

# SOLANA_URL=http://127.0.0.1:8899
SOLANA_URL=https://api.devnet.solana.com
PAYER_KEYPAIR_FILE=payer-keypair.json
PROGRAM_KEYPAIR_FILE=program-keypair.json
ADMIN_KEYPAIR_FILE=admin-keypair.json

# solana-test-validator> /dev/null 2>&1 &
# sleep 10
VALIDATOR_PID=$!
echo "VALIDATOR_PID"
echo $VALIDATOR_PID

while ! curl -X OPTIONS $SOLANA_URL
do
  echo "Waiting 2 seconds"
  sleep 2
done

solana config set --url $SOLANA_URL

echo "KEY_GEN"
yes | solana-keygen new --outfile ./$PAYER_KEYPAIR_FILE
yes | solana-keygen new --outfile ./$PROGRAM_KEYPAIR_FILE
yes | solana-keygen new --outfile ./$ADMIN_KEYPAIR_FILE

echo "AIRDROP"
solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$PAYER_KEYPAIR_FILE 2
solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$PAYER_KEYPAIR_FILE 2
solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$PAYER_KEYPAIR_FILE 2
# solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$PROGRAM_KEYPAIR_FILE 2
solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$ADMIN_KEYPAIR_FILE 2
solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$ADMIN_KEYPAIR_FILE 2
solana airdrop --commitment confirmed --url $SOLANA_URL -k ./$ADMIN_KEYPAIR_FILE 2

./redeploy-devnet.sh $1

echo "DONE"
