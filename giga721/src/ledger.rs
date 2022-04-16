use common::Operation;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{query, update};
use std::cell::RefCell;
use std::rc::Rc;
use crate::guards::{owner_guard, not_paused};

#[cfg(test)]
use crate::testing::time;
#[cfg(not(test))]
use ic_cdk::api::time;

use serde::Serialize;

thread_local! {
    pub static LEDGER: Rc<RefCell<Ledger>> = Rc::new(RefCell::new(Ledger::default()));
}

#[derive(CandidType, Deserialize, Clone, Serialize)]
pub struct Record {
    pub caller: Principal,
    pub op: Operation,
    pub index: u64,
    pub from: Option<Principal>,
    pub to: Option<Principal>,
    pub token_id: u32,
    pub price: Option<u64>,
    pub timestamp: u64,
    pub memo: u64,
}

#[derive(Serialize, CandidType, Deserialize, Default)]
pub struct Ledger {
    pub offset: u64,

    pub storage_canister: Option<Principal>,

    pub tx: Vec<Record>,
}

impl Ledger {
    pub fn get() -> Rc<RefCell<Ledger>> {
        LEDGER.with(|x| x.clone())
    }

    fn add_record(&mut self, record: &Record) {
        self.tx.push((*record).clone());
    }

    fn get_token_history(&mut self, token_id: u32) -> Vec<Record> {
        self.tx.iter().filter(|x| x.token_id == token_id).map(|x| x.clone()).collect()
    }

    // ///Archives records stored in ledger to archive
    // pub async fn archive(&mut self) -> Result<(), String> {
    //     Ok(())
    // }

    //Creates genesis record in ledger canister
    pub fn add_genesis_record(&mut self, caller: Principal) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: caller,
            op: Operation::init,
            from: None,
            to: Some(caller),
            token_id: 0,
            price: None,
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }
    //Creates mint record in ledger
    pub fn mint(&mut self, caller: Principal, owner: Principal, token_id: u32) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: caller,
            op: Operation::mint,
            from: None,
            to: Some(owner),
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }

    /// Adds Burn information to ledger
    #[allow(dead_code)]
    pub fn burn(&mut self, caller: Principal, owner: Principal, token_id: u32) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: caller,
            op: Operation::burn,
            from: Some(owner),
            to: None,
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }

    //Inserts transfer information to ledger
    pub fn transfer(&mut self, from: Principal, to: Principal, token_id: u32) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: from,
            op: Operation::transfer,
            from: Some(from),
            to: Some(to),
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }

    pub fn list(&mut self, from: Principal, token_id: u32, price: u64) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: from,
            op: Operation::list,
            from: Some(from),
            to: None,
            token_id: token_id,
            price: Some(price),
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }

    pub fn delist(&mut self, from: Principal, token_id: u32) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: from,
            op: Operation::delist,
            from: Some(from),
            to: None,
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }

    pub fn purchase(
        &mut self,
        caller: Principal,
        from: Principal,
        to: Principal,
        token_id: u32,
        price: u64,
    ) -> u64 {
        let record = Record {
            index: self.offset + self.tx.len() as u64,
            caller: caller,
            op: Operation::purchase,
            from: Some(from),
            to: Some(to),
            token_id: token_id,
            price: Some(price),
            timestamp: time(),
            memo: 0,
        };

        self.add_record(&record);

        record.index
    }
}

#[query]
pub fn all_history() -> Vec<Record> {
    LEDGER.with(|x| x.borrow().tx.clone())
}

#[query]
pub fn get_history_by_index(index: u128) -> Option<Record> {
    LEDGER.with(|x| {
        // if x.borrow().tx.len() > index as usize { return None; }

        Some(x.borrow().tx[index as usize].clone())
    })
}

#[query]
pub fn get_history_by_token(token: u32) -> Vec<Record> {
    LEDGER.with(|x| x.borrow_mut().get_token_history(token).clone())
    // LEDGER.with(|x| {
    //     x.borrow()
    //         .tx
    //         .iter()
    //         .filter(|y| y.token_id == token)
    //         .collect()
    // })
}

#[query]
pub fn tx_amount() -> u128 {
    LEDGER.with(|x| x.borrow().tx.len() as u128)
}

#[update(guard="owner_guard")]
pub fn upload_history(mut data: Vec<Record>) -> bool {
    LEDGER.with(|x| {
        let mut ledger = x.borrow_mut();
        ledger.tx.append(&mut data);
    });

    true
}