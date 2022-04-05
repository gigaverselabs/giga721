#!/bin/bash

PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx build --network ic token

eval dfx canister --network ic install token --argument="'($NAME, $TICKER, $DESCRIPTION, $SIZE, $PUBLIC_KEY)'" -m upgrade

echo "Upgrade complete"