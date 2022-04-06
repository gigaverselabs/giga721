import { Principal } from "@dfinity/principal";
import { getActor } from "./_common.js";
import 'dotenv/config';
 

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);
  try {

    let creator = Principal.from(process.env.CREATOR);
    let creator_fee = Number(process.env.CREATOR_FEE);

    // console.log(creator.toString());
    // console.log(creator_fee);

    let result1 = await actor.set_creators_address(creator);
    console.log(result1);

    let result2 = await actor.set_creators_fee(creator_fee);
    console.log(result2);

    // let prin = Principal.from('mjfyj-22dca-dcahz-umwwq-vpe4r-iukdj-uuymz-fvphz-rt6my-g7vrs-5qe');
    // let result = await actor.mint_for(1, prin);

    // console.log(result);
  }
  catch (e) {
    console.error(e);
  }
}

run();