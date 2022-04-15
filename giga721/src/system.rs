use crate::ledger::Ledger;
use crate::marketplace::Marketplace;
use crate::storage::StableStorage;
use crate::token::State;

use ic_cdk::export::candid::Principal;
use ic_cdk::print;
use std::collections::HashMap;

use ic_cdk_macros::{init, post_upgrade, pre_upgrade};

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
        is_paused: true,

        tokens: HashMap::default(),
        token_owners: HashMap::default(),
        owners: HashMap::default(),
    };

    *State::get().borrow_mut() = state;

    StableStorage::get().borrow_mut().init_storage().unwrap();
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = State::get();
    let ledger = Ledger::get();
    let market = Marketplace::get();
    let storage = StableStorage::get();
    let mut st = storage.borrow_mut();

    st.store_state((&*state.borrow(), &*ledger.borrow(), &*market.borrow()))
        .unwrap();

    // print(format!("offset: {} , size: {}",st.state_offset, st.state_size));
}

#[post_upgrade]
fn post_upgrade() {
    let storage = StableStorage::get();
    let mut st = storage.borrow_mut();

    st.load_assets().unwrap();

    let (state, ledger, market) =
    st.restore_state().unwrap();

    *State::get().borrow_mut() = state;
    *Ledger::get().borrow_mut() = ledger;
    *Marketplace::get().borrow_mut() = market;
}

#[cfg(test)]
mod test {
    use crate::token::Token;
use crate::token::STATE;
use super::*;
    use crate::storage::STORAGE;

    #[test]
    fn init_test() {
        let prin = crate::testing::user_a();
        init(
            String::from("Name"),
            String::from("Symbol"),
            String::from("Desc"),
            10000,
            prin,
        );
    }

    #[test]
    fn preupgrade_test() {
        let prin = crate::testing::user_a();
        init(
            String::from("Name"),
            String::from("Symbol"),
            String::from("Desc"),
            10000,
            prin,
        );

        pre_upgrade();

        let storage = StableStorage::get();
        let st = storage.borrow_mut();

        // assert_eq!(9, st.state_offset);
        // assert_eq!(343, st.state_size);
    }

    #[test]
    fn postupgrade_test() {
        let prin = crate::testing::user_a();
        init(
            String::from("Name"),
            String::from("Symbol"),
            String::from("Desc"),
            10000,
            prin,
        );

        let asset = crate::testing::get_asset();

        STORAGE.with(|x| {
            for _i in 0..100 {
                x.borrow_mut().store_asset(&asset).unwrap();
            }
        });

        let token = Token {
            id: 0,
            url: String::from("Test"),
            name: String::from("Name"),
            desc: String::from("Desc"),
            properties: Vec::default()
        };

        STATE.with(|x| {
            x.borrow_mut().store_tokens(&vec![token; 10000]);
        });

        pre_upgrade();

        post_upgrade();
    }
}
