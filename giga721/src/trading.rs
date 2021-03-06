use crate::token::State;
use crate::marketplace::{ Listing, Marketplace };
use ic_cdk_macros::{query, update};

use ic_cdk::{caller};

use common::{ TransactionNotification, TransactionResponse };

#[query]
fn get_listed_count() -> u128 {
    let state = Marketplace::get();
    let state = state.borrow();
    return state.listings.len() as u128;
}

// //Returns current listing, by default it is in ascending order
// #[query]
// fn get_listed(page: u128) -> Vec<Listing> {
//     let state = Marketplace::get().borrow();

//     let start = (page*10) as usize;
//     let mut len = 10;

//     if start > state.listings.len() { return vec![]; }

//     if start+len >= state.listings.len() {
//         len = state.listings.len() - start;
//     }

//     return state.listings[start..start+len].to_vec();
// }

//Returns current listing, by default it is in ascending order
#[query]
fn listings() -> Vec<Listing> {
    let state = Marketplace::get();
    let state = state.borrow();
    let vals : Vec<Listing> = state.listings.values().map(|x| x.clone()).collect();
    // let state = get_state();
    // return state.listings.values().map(|x| *x).collect().clone();
    return vals;
}

#[update]
fn list(token_id: u32, price: u64) -> Result<u64, String> {
    //Only token owner can call this
    State::get().borrow().check_owner(token_id, caller())?;
    Marketplace::get().borrow_mut().list(caller(), token_id, price)

}
#[update]
async fn delist(token_id: u32) -> Result<u64, String> {
    //Only token owner can call this
    State::get().borrow().check_owner(token_id, caller())?;
    Marketplace::get().borrow_mut().delist(caller(), token_id)
}

#[update]
async fn transaction_notification(args: TransactionNotification) -> Result<TransactionResponse, String> {
    Marketplace::get().borrow_mut().purchase(caller(), &args)
}