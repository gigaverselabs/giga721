#!/bin/bash

PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx canister --network ic create token
# dfx canister --network ic create ledger_proxy

dfx build --network ic token
# dfx build --network ic ledger_proxy

eval dfx canister --network ic install token --argument="'(\"Infinity Flies\", \"IFNFT\", \"\", 10000, $PUBLIC_KEY)'" -m reinstall
# eval dfx canister --network ic install ledger_proxy -m reinstall

echo "Installation complete"

TOKENID=$(dfx canister --network ic id token)
TOKENID="principal \"$TOKENID\""

LEDGERID=$(dfx canister --network ic id ledger_proxy)
LEDGERID="principal \"$LEDGERID\""

eval dfx canister --network ic call token add_genesis_record
eval dfx canister --network ic call token set_ledger_canister "'($LEDGERID)'"
# eval dfx canister --network ic call ledger_proxy set_token_canister "'($TOKENID)'"
# eval dfx canister --network ic call token set_owner "'(principal \"xm4y3-54lfy-pkijk-3gpzg-gsm3l-yr7al-i5ai7-odpf7-l2pmv-222rl-7qe\")'"
eval dfx canister --network ic call token set_owner "'(principal \"vwm6j-rqaaa-aaaah-qclba-cai\")'"

echo "Preparation complete"
