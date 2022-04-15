use crate::token::{Token, TokenOwner};
use crate::storage::STORAGE;
use crate::token::STATE;
use crate::storage::Asset;

use ic_cdk::{caller};
use ic_cdk_macros::{update};
use ic_cdk::export::candid::{Principal};

use crate::guards::{owner_guard};

#[update(guard="owner_guard")]
fn mint_for(token_id: u128, owner: Principal) -> Result<u64, String> {
    STATE.with(|x| x.borrow_mut().mint_token_id(caller(), owner, token_id as u32))
}

#[update]
fn burn(token_id: u128) -> Result<u64, String> {
    STATE.with(|x| x.borrow_mut().burn(caller(), token_id as u32))
}

// #[update(guard="owner_guard")]
// async fn multi_mint(data: Vec<MintRequest>) -> Vec<u128> {
//     let state = get_state();

//     if (state.tokens.len()+data.len()) as u32 > state.max_supply {trap("Max token count reached");}

//     let mut result : Vec<u128> = Vec::with_capacity(data.len());

//     let owner = caller().clone();

//     for item in data {
//         match state.mint(item, owner).await {
//             Ok(id) => result.push(id),
//             Err(_) => {}
//         }        
//     }

//     return result;
// }

#[update(guard="owner_guard")]
fn upload_asset(data: Asset) -> Result<(), String> {
    STORAGE.with(|x| {
        x.borrow_mut().store_asset(&data)
    })
}

//Uploads metadata of given token
#[update(guard="owner_guard")]
fn upload_tokens_metadata(_data: Vec<Token>) -> Result<(), String> {
    STATE.with(|x| {
        x.borrow_mut().store_tokens(&_data);
    });

    Ok(())
}