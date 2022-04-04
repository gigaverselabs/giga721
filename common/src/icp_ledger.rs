use ic_cdk::export::candid::{Decode, Principal, encode_args};
use ic_cdk::api::call::{call_raw};

use crate::types::SendArgs;


pub async fn call_send_dfx(canister: Principal, args: &SendArgs) -> Result<u64, String> {
    //Encode args in candid
    let event_raw = encode_args((args,))
        .map_err(|_| String::from("Cannot serialize Transaction Args"))?;

    //Inter container call to ledger canister
    let raw_res = call_raw(canister, "send_dfx", event_raw.clone(), 0)
        .await
        .map_err(|(_, s)| format!("Error invoking Ledger Canister, {}", &s))?;

    // //Todo: deserialize send_dfx result to get block height!
    let res = Decode!(&raw_res, u64)
        .map_err(|_| String::from("Error decoding response from Ledger canister"))?;

    Ok(res)
}