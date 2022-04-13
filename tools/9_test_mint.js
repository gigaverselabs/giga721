import { Principal } from "@dfinity/principal";
import { getActor } from "./_common.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {
    let prin = Principal.from('mjfyj-22dca-dcahz-umwwq-vpe4r-iukdj-uuymz-fvphz-rt6my-g7vrs-5qe');
    let result = await actor.mint_for(1, prin);

    console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();