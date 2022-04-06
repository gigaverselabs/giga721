import { getActor } from "./_common.js";
import { getMetadata } from "./_data_converter.js";

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function run() {
  let actor = getActor(true);


  let data = getMetadata();

  try {
    let result = await actor.upload_tokens_metadata(data);

    console.log(result);

    let data_res = await actor.metadata();

    console.log(data_res.length);
  }
  catch (e) {
    console.error(e);
  }
}

run();