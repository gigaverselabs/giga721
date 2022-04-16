import { Principal } from '@dfinity/principal';
import fs from 'fs';

export function getAirdrop() {
  let metadata = JSON.parse(fs.readFileSync('../../airdrop.json'));

  let data = [];

  let start = 1;

  for (var i = 0; i < metadata.length; i++) {
    let prin = metadata[i]["Principal ID"];
    let num = metadata[i]["Number to Send"];

    Principal.from(prin);

    for (var n=0;n<num;n++) {
      data.push({
        no: start++,
        principal: prin
      });
    }


    // let item = convert_data(metadata[i]);

    // data.push(item);
  }

  return data;
}