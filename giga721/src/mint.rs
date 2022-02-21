use crate::token::Token;
use crate::storage::STORAGE;
use crate::token::STATE;
use crate::storage::Asset;

use ic_cdk::{caller};
use ic_cdk_macros::{update};

use crate::guards::{owner_guard};
// use crate::state::get_state;




#[update(guard="owner_guard")]
async fn mint() -> Result<u32, String> {
    // let state = get_state();

    STATE.with(|x| x.borrow_mut().mint(caller()))

    // match state.mint(data, caller()).await {
    //     Ok(id) => return id,
    //     Err(s) => trap(&s),
    // }
}
#[update(guard="owner_guard")]
async fn mint_id(token_id: u32) -> Result<u32, String> {
    // let state = get_state();

    STATE.with(|x| x.borrow_mut().mint(caller()))

    // match state.mint(data, caller()).await {
    //     Ok(id) => return id,
    //     Err(s) => trap(&s),
    // }
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
async fn upload_asset(data: Asset) -> Result<(), String> {
    STORAGE.with(|x| {
        x.borrow_mut().store_asset(&data)
    })
}

//Uploads metadata of given token
#[update(guard="owner_guard")]
async fn upload_tokens_metadata(_data: Vec<Token>) -> Result<(), String> {
    
    
    Ok(())
}