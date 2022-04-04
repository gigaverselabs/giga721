#!/bin/bash

PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx canister --network ic create token

dfx build --network ic token

eval dfx canister --network ic install token --argument="'(\"Infinity Flies\", \"IFNFT\", \"\", 10000, $PUBLIC_KEY)'"

echo "Installation complete"

eval dfx canister --network ic call token add_genesis_record

if ! [ -z ${OWNER+x} ]; then
    echo "Setting canister owner"
    eval dfx canister --network ic call token set_owner "'(principal $OWNER)'"
fi

echo "Preparation complete"
