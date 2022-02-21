use std::rc::Rc;
use std::cell::RefCell;
use common::{Operation};
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

#[cfg(test)]
use crate::testing::{time};

use serde::Serialize;

#[cfg(not(test))]
use ic_cdk::api::time;


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
    pub memo: u64
}

#[derive(Serialize, CandidType, Deserialize, Default)]
pub struct Ledger {
    pub offset: u64,
    pub tx: Vec<Record>
}

impl Ledger {
    pub fn get() -> Rc<RefCell<Ledger>> {
        LEDGER.with(|x| x.clone())
    }

    fn add_record(&mut self, record: &Record) {
        self.tx.push((*record).clone());
    }

    //Creates genesis record in ledger canister
    pub fn add_genesis_record(&mut self, caller: Principal) -> u64 {
        let record = Record {
            index: self.offset+self.tx.len() as u64,
            caller: caller,
            op: Operation::init,
            from: None,
            to: Some(caller),
            token_id: 0,
            price: None,
            timestamp: time(),
            memo: 0
        };

        self.add_record(&record);

        record.index

        // let owner = caller();
        
        // let event_raw = ic_cdk::export::candid::encode_args((
        //     owner, //caller 
        //     Operation::init, //op 
        //     None::<Principal>, //from
        //     owner, //to
        //     0u128, //tokenId
        //     None::<u64>, //price
        //     time() as i128 //timestamp
        // )).unwrap();
        
        // let res = ic_cdk::api::call::call_raw(
        //     self.storage_canister.unwrap(),
        //     "addRecord",
        //     event_raw.clone(),
        //     0,
        // ).await;

        // match res {
        //     Ok(res) =>{
        //         let val1 = Decode!(&res, u128).unwrap();
        //         return Ok(val1);
        //     },
        //     Err((_, s)) => {
        //         return Err(s);
        //     }
        // }
    }

    pub fn mint(&mut self, caller: Principal, owner: Principal, token_id: u32) -> u64 {
        let record = Record {
            index: self.offset+self.tx.len() as u64,
            caller: caller,
            op: Operation::mint,
            from: Some(owner),
            to: None,
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0
        };

        self.add_record(&record);

        record.index
    }

    pub fn transfer(&mut self, from: Principal, to: Principal, token_id: u32) -> u64{
        let record = Record {
            index: self.offset+self.tx.len() as u64,
            caller: from,
            op: Operation::mint,
            from: Some(from),
            to: Some(to),
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0
        };

        self.add_record(&record);

        record.index
    }

    pub fn list(&mut self, from: Principal, token_id: u32, price: u64) -> u64 {
        let record = Record {
            index: self.offset+self.tx.len() as u64,
            caller: from,
            op: Operation::list,
            from: Some(from),
            to: None,
            token_id: token_id,
            price: Some(price),
            timestamp: time(),
            memo: 0
        };

        self.add_record(&record);

        record.index
    }

    pub fn delist(&mut self, from: Principal, token_id: u32) -> u64 {
        let record = Record {
            index: self.offset+self.tx.len() as u64,
            caller: from,
            op: Operation::delist,
            from: Some(from),
            to: None,
            token_id: token_id,
            price: None,
            timestamp: time(),
            memo: 0
        };

        self.add_record(&record);

        record.index
    }

    pub fn purchase(&mut self, caller: Principal, from: Principal, to: Principal, token_id: u32, price: u64) -> u64 {
        let record = Record {
            index: self.offset+self.tx.len() as u64,
            caller: caller,
            op: Operation::purchase,
            from: Some(from),
            to: Some(to),
            token_id: token_id,
            price: Some(price),
            timestamp: time(),
            memo: 0
        };

        self.add_record(&record);

        record.index 

    }
}