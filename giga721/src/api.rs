use crate::marketplace::MARKETPLACE;
use crate::token::State;
use crate::marketplace::Marketplace;
use crate::ledger::LEDGER;
use crate::token::STATE;
use ic_cdk::{caller, trap};
use ic_cdk_macros::{query, update};

use ic_cdk::export::candid::Principal;

use crate::token::{Token, TokenDesc, TokenOwner};

use crate::guards::{owner_guard};

#[query]
fn get_ledger_canister() -> Option<Principal> {
    Marketplace::get().borrow().ledger_canister
}
#[query]
fn get_cycles() -> u128 {
   return ic_cdk::api::canister_balance() as u128;
}

#[query]
fn tx_enabled() -> bool {
    Marketplace::get().borrow().tx_enabled
}
#[query]
fn is_paused() -> bool {
    State::get().borrow().is_paused
}

#[update(guard="owner_guard")]
fn set_tx_enabled(enabled: bool) -> bool {
    Marketplace::get().borrow_mut().tx_enabled = enabled;

    return true;
}

#[update(guard="owner_guard")]
fn set_paused(paused: bool) -> bool {
    State::get().borrow_mut().is_paused = paused;

    return true;
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
}

#[update(guard="owner_guard")]
fn set_creators_fee(fee: u128) -> bool {
    MARKETPLACE.with(|x| x.borrow_mut().creators_fee = fee);    

    return true;
}

#[query]
fn creators_address() -> Option<Principal> {
    Marketplace::get().borrow().creators_address
}

#[update(guard="owner_guard")]
fn set_creators_address(creator: Principal) -> bool {
    MARKETPLACE.with(|x| x.borrow_mut().creators_address = Some(creator));    

    return true;
}

#[query]
fn owners() -> Vec<TokenOwner> {
    STATE.with(|x| x.borrow_mut().owners())
}

#[query]
fn owner_of(token_id: u128) -> Principal {
    let state = State::get();
    let mut state = state.borrow_mut();

    state.get_owner(token_id as u32).ok().unwrap()
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

#[query]
fn data_of(token_id: u128) -> TokenDesc {
    let result = STATE.with(|x| x.borrow_mut().data_of(token_id as u32).map_err(|s| trap(&s)));    

    result.unwrap()
}

#[query]
fn metadata() -> Vec<Token> {
    STATE.with(|x| x.borrow().tokens.values().map(|x| (*x).clone()).collect())
}


#[query]
fn tokens() -> Vec<u32> {
    // let state = State::get();
    // return state.tokens.clone();

    STATE.with(|x| {
        let tokens : Vec<u32> = x.borrow().token_owners.keys().map(|x| *x).collect();

        tokens
    })
}

#[update(guard="owner_guard")]
fn set_owner(owner: Principal) -> bool {
    STATE.with(|x| x.borrow_mut().owner = Some(owner));    

    return true;
}
#[update(guard="owner_guard")]
fn set_description(description: String) -> bool {
    STATE.with(|x| x.borrow_mut().description = description);    

    return true;
}
#[update(guard="owner_guard")]
fn set_icon_url(icon_url: String) -> bool {
    STATE.with(|x| x.borrow_mut().icon_url = icon_url);    

    return true;
}


#[update(guard="owner_guard")]
async fn add_genesis_record() -> Result<u64, String> {
    Ok(LEDGER.with(|x| x.borrow_mut().add_genesis_record(caller())))
}

#[update(guard="owner_guard")]
fn set_ledger_canister(ledger: Principal) -> bool {
    Marketplace::get().borrow_mut().ledger_canister = Some(ledger);

    return true;
}

//Used to transfer @token_id from owner to @to
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