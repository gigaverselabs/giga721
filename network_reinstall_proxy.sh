#!/bin/bash

PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

. ./variables.sh

dfx canister --network ic create ledger_proxy

dfx build --network ic ledger_proxy

eval dfx canister --network ic install ledger_proxy -m reinstall

echo "Installation complete"

TOKENID=$(dfx canister --network ic id token)
TOKENID="principal \"$TOKENID\""

LEDGERID=$(dfx canister --network ic id ledger_proxy)
LEDGERID="principal \"$LEDGERID\""

# eval dfx canister --network ic call token set_ledger_canister "'($LEDGERID)'"
eval dfx canister --network ic call ledger_proxy set_token_canister "'($TOKENID)'"

echo "Preparation complete"
