import { Actor, HttpAgent } from '@dfinity/agent';
import { Ed25519KeyIdentity } from '@dfinity/identity';

import { idlFactory } from '../.dfx/ic/canisters/token/token.did.js';
import { idlFactory as proxy_idlFactory } from '../.dfx/ic/canisters/ledger_proxy/ledger_proxy.did.js';
import fetch from 'node-fetch';
import fs from 'fs';

global.fetch = fetch;

var keyData = fs.readFileSync('./key.json', 'utf8');
var key = Ed25519KeyIdentity.fromJSON(keyData);
console.log("Loaded principal: " + key.getPrincipal().toString())

export function getCanisterId(useProd, canister) {
    let canisterId = null;

    if (useProd) {
        var data = JSON.parse(fs.readFileSync("../canister_ids.json"))
        canisterId = data[canister]["ic"];

    } else {
        var data = JSON.parse(fs.readFileSync("../.dfx/local/canister_ids.json"))
        canisterId = data[canister]["local"];
    }

    console.log(canister+" Canister Id: " + canisterId);
    return canisterId;
}

//Returns actor for token canister
export function getActor(useProd) {

    let httpAgent = null;
    let canisterId = getCanisterId(useProd, "token");

    if (useProd) {
        var host = "https://boundary.ic0.app/"; //ic

        httpAgent = new HttpAgent({ identity: key, host });
    } else {
        const host = "http://127.0.0.1:8000"; //local

        httpAgent = new HttpAgent({ identity: key, host });
        httpAgent.fetchRootKey();
    }

    const actor = Actor.createActor(idlFactory, {
        agent: httpAgent,
        canisterId: canisterId,
    });

    return actor;
}

//Returns actor for token canister
export function getLedgerActor(useProd) {

    let httpAgent = null;
    let canisterId = getCanisterId(useProd, "ledger_proxy");

    if (useProd) {
        var host = "https://boundary.ic0.app/"; //ic

        httpAgent = new HttpAgent({ identity: key, host });
    } else {
        const host = "http://127.0.0.1:8000"; //local

        httpAgent = new HttpAgent({ identity: key, host });
        httpAgent.fetchRootKey();
    }

    const actor = Actor.createActor(proxy_idlFactory, {
        agent: httpAgent,
        canisterId: canisterId,
    });

    return actor;
}