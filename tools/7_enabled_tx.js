import { getActor } from "./_common.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {
    let result = await actor.set_tx_enabled(true);

    console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();