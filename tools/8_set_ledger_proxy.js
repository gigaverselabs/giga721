import { Principal } from "@dfinity/principal";
import { getActor } from "./_common.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {
    let canister = Principal.from('i3oug-lyaaa-aaaah-qco3a-cai');
    let result = await actor.set_ledger_canister(canister);

    console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();