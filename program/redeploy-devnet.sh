#!/bin/bash

set -e

PAYER_KEYPAIR_FILE=payer-keypair.json
PROGRAM_KEYPAIR_FILE=program-keypair.json
ADMIN_KEYPAIR_FILE=admin-keypair.json

echo "PROGRAM COMPILE"
export ADMIN_PUBKEY=$(cd interface && npx ts-node scripts/get-public-key.ts ../$ADMIN_KEYPAIR_FILE)
echo "ADMIN PUBKEY: $ADMIN_PUBKEY"
cargo build-bpf

echo "PROGRAM DEPLOY"
PROGRAM_ID=$(solana program deploy \
--commitment confirmed \
-k ./$PAYER_KEYPAIR_FILE \
./target/deploy/cwar_token_staking.so \
--program-id ./$PROGRAM_KEYPAIR_FILE)

PROGRAM_ID=$(echo "$PROGRAM_ID" | tail -n1)
export SOLANA_PROGRAM_ID=${PROGRAM_ID#"Program Id: "}
echo $PROGRAM_ID
cd interface
ts-node ./scripts/initialize-pool.ts $1
# solana program show "$PROGRAM_ID"
