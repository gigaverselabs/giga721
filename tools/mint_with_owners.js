import { Actor, HttpAgent } from '@dfinity/agent';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { Principal } from '@dfinity/principal';
import fetch from 'node-fetch';
import {
  idlFactory
} from "./icpunks.js";

import fs from 'fs';
import process from 'process';

// import { readFile } from 'fs/promises';
// const traits = JSON.parse(
//   await readFile(
//     new URL('../data/finaltraits.json', import.meta.url)
//   )
// );

global.fetch = fetch;

var keyData = fs.readFileSync('./key.json', 'utf8');
var key = Ed25519KeyIdentity.fromJSON(keyData);

//specify localhost endpoint or ic endpoint;
const host = "https://boundary.ic0.app/"; //ic
var canister_id = "nvtz2-maaaa-aaaah-qcohq-cai";
var canister_id_storage = "pioxs-7iaaa-aaaah-qcoia-cai";
// const host = "http://127.0.0.1:8000"; //local
// var canister_id = "rwlgt-iiaaa-aaaaa-aaaaa-cai";

const http = new HttpAgent({ identity: key, host });
http.fetchRootKey();

console.log("Loaded principal: " + key.getPrincipal().toString())

const actor = Actor.createActor(idlFactory, {
  agent: http,
  canisterId: canister_id,
});

const storage_actor = Actor.createActor(idlFactory_storage, {
  agent: http,
  canisterId: canister_id_storage,
});

let ownerPrincipal = Principal.fromText("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe")

//Prepares mint request using provided data
function make_request(trait) {
  var [imagePath, contentType] = get_image_path(trait.tokenId);

  // var buffer = fs.readFileSync(imagePath);
  // var data = [...buffer];

  var data = [];

  var props = [];

  for (let p in trait.props) {
    props.push({ name: p, value: trait.props[p] });
  }

  var mintRequest = {
    url: "/Token/" + (trait.tokenId),
    content_type: "",
    desc: "",
    name: "ICTest #" + (trait.tokenId),
    data: data,
    properties: props,
    owner: ownerPrincipal
  };

  return mintRequest;
}

function get_image_path() {
  var path = '../data/image';

  if (fs.existsSync(path + '.jpg')) return [path + '.jpg', 'image/jpg'];
  if (fs.existsSync(path + '.png')) return [path + '.png', 'image/png'];

  return [];
}

//Mints new tokens using multi_mint feature, before sending package of tokens to mint, checks if the request is within max_size limits (currently 2mb of data)
async function multi_mint() {
  var total_minted = 0;
  var hrstart = process.hrtime()
  var traits_length = traits.length;

  try {
    var multi_mint = [];

    for (let i=0;i<traits_length;i++) {
      let item = make_request(traits[i]);
      multi_mint.push(item);
    }

    console.log('Minting tokens: ' + multi_mint.length + ' ' + total_minted);
    await actor.multi_mint(multi_mint);
    total_minted += multi_mint.length;

  } catch (e) {
    debugger
  }

  var hrend = process.hrtime(hrstart)
  console.log("Uploading " + traits.length + " tokens took : %ds %dms", hrend[0], hrend[1] / 1000000);
}

multi_mint();