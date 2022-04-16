import { Principal } from "@dfinity/principal";
import { getActor, getCanisterId } from "./_common.js";
import 'dotenv/config';
 

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {

    let creator = Principal.from(process.env.CREATOR);
    let creator_fee = Number(process.env.CREATOR_FEE);

    console.log("Creator: "+creator.toString());
    console.log("Fee: "+creator_fee);

    let canister = Principal.from(getCanisterId(true, "ledger_proxy"));
    let result = await actor.set_ledger_canister(canister);
    console.log(result);

    let result1 = await actor.set_creators_address(creator);
    console.log(result1);

    let result2 = await actor.set_creators_fee(creator_fee);
    console.log(result2);

    let result3 = await actor.set_tx_enabled(true);
    console.log(result3);

    let result4 = await actor.set_paused(false);
    console.log(result4);
  }
  catch (e) {
    console.error(e);
  }
}

run();