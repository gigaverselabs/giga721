use crate::state::NotificationStatus;
use crate::state::ProxyStatus;
use crate::state::TransferStatus;
use ic_cdk::export::candid::Principal;
use ic_cdk_macros::*;

use common::SendArgs;

mod ledger {
    include!("../gen/ic_ledger.pb.v1.rs");
}

mod int_set;

mod storage;
use crate::storage::StableStorage;

mod state;
use state::{State, STATE};

#[init]
fn init() {
    StableStorage::get().borrow_mut().init_storage().unwrap();
}

#[pre_upgrade]
fn pre() {
    let storage = StableStorage::get();
    let mut st = storage.borrow_mut();

    STATE.with(|state| {
        st.store_state(&*state.borrow()).unwrap();
    });
}

#[post_upgrade]
fn post() {
    let storage = StableStorage::get();
    let mut st = storage.borrow_mut();

    st.load_assets().unwrap();

    let state = st.restore_state().unwrap();

    STATE.with(|x| {
        *x.borrow_mut() = state;
    });

    // let (st, ) : (State, ) = ic_cdk::storage::stable_restore().unwrap();

    // STATE.with(|state| {
    //     *state.borrow_mut() = st;

    // })
}

#[update]
fn set_ledger_canister(id: Principal) -> bool {
    STATE.with(|s| {
        s.borrow_mut().ledger_canister = Some(id);
    });

    true
}

#[update]
fn set_token_canister(id: Principal) -> bool {
    STATE.with(|s| {
        s.borrow_mut().token_canister = Some(id);
    });

    true
}

#[query]
fn get_blocks() -> Vec<SendArgs> {
    STATE.with(|s| s.borrow().blocks.clone())
}

#[query]
fn count_processed() -> u64 {
    STATE.with(|s| s.borrow().blocks_processed.len() as u64)
}

#[query]
fn get_processed() -> Vec<u64> {
    STATE.with(|s| s.borrow().blocks_processed.keys().map(|x| *x).collect())
}

#[query]
fn get_payments() -> Vec<TransferStatus> {
    STATE.with(|s| s.borrow().payments.clone())
}

#[query]
fn get_notifications() -> Vec<NotificationStatus> {
    STATE.with(|s| s.borrow().notifications.clone())
}

#[query]
fn get_market_fee() -> u64 {
    STATE.with(|s| s.borrow().market_fee)
}

#[query]
fn get_status() -> ProxyStatus {
    STATE.with(|s| ProxyStatus {
        total_market_fee: s.borrow().total_market_fee,
        total_creator_fee: s.borrow().total_creator_fee,
        waiting_market_fee: s.borrow().waiting_market_fee,
        waiting_creator_fee: s.borrow().waiting_creator_fee,
    })
}

//Notifies canister about transaction
#[update]
async fn notify(block_height: u64) -> Result<(), String> {
    let state = STATE.with(|s| s.clone());
    let mut state = state.borrow_mut();

    state.notify(block_height).await
}
