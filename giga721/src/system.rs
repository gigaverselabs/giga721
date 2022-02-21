use crate::token::State;
use crate::ledger::Ledger;
use crate::marketplace::Marketplace;
use crate::storage::StableStorage;

use std::collections::HashMap;
use ic_cdk::export::candid::{Principal};


use ic_cdk_macros::{init, pre_upgrade, post_upgrade};


#[cfg(test)]
use crate::testing::{stable_read, stable_write, stable_size, stable_grow};

#[cfg(not(test))]
use ic_cdk::api::stable::{stable_read, stable_write, stable_size, stable_grow};


#[init]
fn init(name: String, symbol: String, desc: String, max_supply: i128, owner: Principal) {
    let state = State {
        owner: Some(owner),
        name: name.clone(),
        symbol: symbol.clone(),
        description: desc.clone(),
        icon_url: "None".to_string(),

        max_supply: max_supply as u32,
        total_supply: 0,

        tokens: Vec::with_capacity(max_supply as usize),
        token_owners: HashMap::default(),
        owners: HashMap::default()
    };

    *State::get().borrow_mut() = state;

    StableStorage::get().borrow_mut().init_storage();
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = State::get();
    let ledger = Ledger::get();
    let market = Marketplace::get();
    let storage = StableStorage::get();

    let mut st = storage.borrow_mut();

    st.store_state((
            &*state.borrow(),
            &*ledger.borrow(),
            &*market.borrow()
        ));
}

#[post_upgrade]
fn post_upgrade() {
    let storage = StableStorage::get();
    let mut st = storage.borrow_mut();

    let (state, ledger, market) =
    st.restore_state().ok().unwrap();

    *State::get().borrow_mut() = state;
    *Ledger::get().borrow_mut() = ledger;
    *Marketplace::get().borrow_mut() = market;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_test() {
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();
        init(String::from("Name"), String::from("Symbol"), String::from("Desc"), 10000, prin);
    }

    #[test]
    fn preupgrade_test() {
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();
        init(String::from("Name"), String::from("Symbol"), String::from("Desc"), 10000, prin);

        pre_upgrade();

        let storage = StableStorage::get();
        let mut st = storage.borrow_mut();

        assert_eq!(9, st.state_offset);
        assert_eq!(467, st.state_size);
    }

    #[test]
    fn postupgrade_test() {
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();
        init(String::from("Name"), String::from("Symbol"), String::from("Desc"), 10000, prin);

        pre_upgrade();

        // post_upgrade();

        let storage = StableStorage::get();
        let mut st = storage.borrow_mut();

        let (state, ledger, market) = st.restore_state().unwrap();
        // let restore = st.restore_state();

        *State::get().borrow_mut() = state;
        *Ledger::get().borrow_mut() = ledger;
        *Marketplace::get().borrow_mut() = market;

        // assert_eq!(restore, Ok(()));

        assert_eq!(9, st.state_offset);
        assert_eq!(265, st.state_size);
    }
}