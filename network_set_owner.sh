#!/bin/bash

. ./variables.sh

if ! [ -z ${OWNER+x} ]; then
    echo "Setting canister owner"
    eval dfx canister --network ic call token set_owner "'(principal $OWNER)'"
fi
