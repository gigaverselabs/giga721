import { Principal } from "@dfinity/principal";
import { getActor } from "./_common.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {
    let prin = Principal.from('dwymk-kn72k-3b7pm-jkqo6-w2b6o-mb4wc-amwya-k2m4s-7vh54-qq5p3-kqe');
    let result = await actor.mint_for(3, prin);

    console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();