import { Principal } from "@dfinity/principal";
import { getActor, getLedgerActor } from "./_common.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let ledger_actor = getLedgerActor(true);
  try {
    let result = await ledger_actor.notify(3162641);

    console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();