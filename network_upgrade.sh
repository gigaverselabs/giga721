#!/bin/bash

PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx canister --network ic create token
# dfx canister --network ic create ledger_proxy

dfx build --network ic token
# dfx build --network ic ledger_proxy

eval dfx canister --network ic install token --argument="'(\"Infinity Flies\", \"IFNFT\", \"\", 10000, $PUBLIC_KEY)'" -m upgrade
# eval dfx canister --network ic install ledger_proxy -m upgrade

echo "Upgrade complete"