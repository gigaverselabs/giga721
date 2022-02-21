use crate::token::State;
use crate::marketplace::Marketplace;
use crate::ledger::LEDGER;
use crate::token::STATE;
use ic_cdk::{caller, trap};
use ic_cdk_macros::{query, update};

use ic_cdk::export::candid::Principal;

use crate::token::{Token, TokenDesc, TokenDescExt};

// use crate::state::get_state;
use crate::guards::{owner_guard};

#[query]
fn get_cycles() -> u128 {
   return ic_cdk::api::canister_balance() as u128;
}


// #[query]
// fn get_storage_canister() -> Option<Principal> {
//     State::get().borrow().storage_canister;
// }

#[query]
fn get_ledger_canister() -> Option<Principal> {
    Marketplace::get().borrow().ledger_canister
}

#[query]
fn name() -> String {
    State::get().borrow().name.clone()
}

#[query]
fn symbol() -> String {
    State::get().borrow().symbol.clone()
}

#[query]
fn description() -> String {
    State::get().borrow().description.clone()
}
#[query]
fn icon_url() -> String {
    State::get().borrow().icon_url.clone()
}

#[query]
fn owner() -> Principal {
    State::get().borrow().owner.unwrap()
}

#[query]
fn total_supply() -> u128 {
    State::get().borrow().total_supply as u128
}
#[query]
fn creators_fee() -> u128 {
    Marketplace::get().borrow().creators_fee
    // return State::get().creators_fee;
}

#[query]
fn tx_enabled() -> bool {
    Marketplace::get().borrow().tx_enabled
}

#[update(guard="owner_guard")]
fn set_tx_enabled(enabled: bool) -> bool {
    // let state = State::get();
    // state.tx_enabled = enabled;
    Marketplace::get().borrow_mut().tx_enabled = enabled;

    return true;
}

#[query]
fn owner_of(token_id: u128) -> Option<Principal> {
    // let state = State::get();
    // if token_id > (state.tokens.len() as u128) || token_id == 0 {trap("Invalid token_id");}

    // let pos = (token_id as usize)-1;
    // return state.tokens.get(pos).unwrap().owner;

    let state = State::get();
    let mut state = state.borrow_mut();

    state.get_owner(token_id as u32).ok()
}

#[query]
fn user_tokens(user: Principal) -> Vec<u128> {
    let state = State::get();
    let state = state.borrow();

    let list = state.owners.get(&user);

    match list {
        Some(list) => {
            return list.clone();
        },
        None => {
            return vec![];
        }
    }
}

// #[query]
// fn data_of(token_id: u128) -> TokenDesc {
//     let state = State::get();
//     if token_id > (state.tokens.len() as u128) || token_id == 0 {trap("Invalid token_id");}
//     let pos = (token_id as usize)-1;

//     return state.tokens[pos].clone();
// }
// #[query]
// fn tokens() -> Vec<TokenDesc> {
//     // let state = State::get();
//     // return state.tokens.clone();

//     STATE.with(|x| x.borrow().getTokenDesc().clone())
// }

#[update(guard="owner_guard")]
fn set_owner(owner: Principal) -> bool {
    // let state = State::get();
    // state.owner = owner;

    STATE.with(|x| x.borrow_mut().owner = Some(owner));    

    return true;
}
#[update(guard="owner_guard")]
fn set_description(description: String) -> bool {
    // let state = State::get();
    // state.description = description;

    STATE.with(|x| x.borrow_mut().description = description);    

    return true;
}
#[update(guard="owner_guard")]
fn set_icon_url(icon_url: String) -> bool {
    // let state = State::get();
    // state.icon_url = icon_url;

    // return true;
    STATE.with(|x| x.borrow_mut().icon_url = icon_url);    

    return true;
}


#[update(guard="owner_guard")]
async fn add_genesis_record() -> Result<u64, String> {
    // let state = State::get();

    // if state.storage_canister == None {trap("Storage Canister is null");}

    // match state.add_genesis_record().await {
        // Ok(id) => return id,
        // Err(s) => trap(&s),
    // }
    Ok(LEDGER.with(|x| x.borrow_mut().add_genesis_record(caller())))
}

// #[update(guard="owner_guard")]
// fn set_storage_canister(storage: Principal) -> bool {
//     State::get().storage_canister = Some(storage);

//     return true;
// }

#[update(guard="owner_guard")]
fn set_ledger_canister(ledger: Principal) -> bool {
    Marketplace::get().borrow_mut().ledger_canister = Some(ledger);

    return true;
}

#[update]
async fn transfer_to(to: Principal, token_id: u128) -> bool {
    let res = STATE.with(|x| x.borrow_mut().transfer(caller(), to, token_id as u32));

    match res {
        Ok(_) => return true,
        Err(s) => trap(&s)
    }
}

// #[update]
// async fn multi_transfer_to(data: Vec<(Principal, u128)>) -> Vec<bool> {
//     let state = State::get();
//     //Check if tokenId is valid

//     let mut result : Vec<bool> = Vec::with_capacity(data.len());

//     for item in data {
//         match state.transfer(caller(), item.0, item.1).await {
//             Ok(id) => result.push(id),
//             Err(_) => {}
//         }        
//     }

//     return result;
// }