import { getAirdrop } from "./_drop_converter.js";
import { getActor } from "./_common.js";
import { Principal } from "@dfinity/principal";

async function run() {
    let tokens = getAirdrop();

    let actor = getActor(true);

    let batch_size = 50;
    let wait = [];
    
    for (let x in tokens) {
        let token_no = tokens[x].no;

        try {
            let principal = Principal.fromText(tokens[x].principal);
            console.log("Minting token: " + token_no + " to: " + principal.toString());
            wait.push(actor.mint_for(token_no, principal));
        } catch (e) {
            console.log(e);
        }

        if (wait.length >= batch_size) {
            let transfer_result = await Promise.all(wait);
            // console.log("Result: " + JSON.stringify(transfer_result));
            wait = [];
        }
    }

    let transfer_result = await Promise.all(wait);
    // debugger;
    // console.log("Result: " + JSON.stringify(transfer_result));
}

run();