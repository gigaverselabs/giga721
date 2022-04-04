#!/bin/bash

# dfx stop
# rm -r .dfx/local

# dfx start --background --clean 
PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx canister --no-wallet create token
# dfx canister --no-wallet create ledger_proxy

dfx build token
# dfx build ledger_proxy

eval dfx canister --no-wallet install token --argument="'(\"ICTest\", \"ICT\", \"\", 10000, $PUBLIC_KEY)'" -m upgrade