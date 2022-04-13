import { Principal } from "@dfinity/principal";
import { getActor } from "./_common.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {
    let fee_result = await actor.set_creators_fee(2500);
    console.log(result);

    let addr_result = await actor.set_creators_address(Principal.from('k6d5p-bu67j-vvzcu-n4pr7-al5gn-tkm4z-pq3by-b4ehr-n36sp-z66dp-mqe'));
    console.log(result);

    let result = await actor.set_tx_enabled(true);
    console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();