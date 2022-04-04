#!/bin/bash

# dfx stop
# rm -r .dfx/local

# dfx start --background --clean 
PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx canister --no-wallet create token
dfx canister --no-wallet create storage

dfx build token
dfx build storage

eval dfx canister --no-wallet install token --argument="'(\"ICTest\", \"ICT\", \"\", 10000, $PUBLIC_KEY)'"
eval dfx canister --no-wallet install storage --argument="'($PUBLIC_KEY)'"

TOKENID=$(dfx canister --no-wallet id token)
STOREID=$(dfx canister --no-wallet id storage)

TOKENID="principal \"$TOKENID\""
STOREID="principal \"$STOREID\""

eval dfx canister --no-wallet call token set_storage_canister "'($STOREID)'"
eval dfx canister --no-wallet call storage setTokenCanisterId "'($TOKENID)'"
eval dfx canister --no-wallet call token add_genesis_record

eval dfx canister --no-wallet call token set_owner "'(principal \"k3r3y-gsxlr-4jp3j-vvyk3-jnux2-7da37-muovr-7xphw-2v2wd-2hvms-sqe\")'"