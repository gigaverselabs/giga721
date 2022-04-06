#!/bin/bash

PUBLIC_KEY="principal \"$( \
    dfx identity get-principal
)\""

dfx build --network ic ledger_proxy

eval dfx canister --network ic install ledger_proxy -m upgrade