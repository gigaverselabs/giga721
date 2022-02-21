use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use common::{Property, HeaderField};

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use crate::ledger::{LEDGER};
use crate::marketplace::{MARKETPLACE};

use serde::Serialize;


thread_local! {
    pub static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State::default()));
}

// pub fn get_state() -> RefCell<State> {
//     *STATE.with(|x| x)
// }

#[derive(CandidType, Deserialize)]
pub struct OldState {
    pub owner: Principal,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub icon_url: String,
    pub max_supply: u32,

    pub storage_canister: Option<Principal>,
    pub ledger_canister: Option<Principal>,

    pub tokens: Vec<Token>,
    pub owners: HashMap<Principal, Vec<u128>>,
    pub assets: HashMap<String, (Vec<HeaderField>, Vec<u8>)>,
}

#[derive(CandidType, Deserialize, Clone, Serialize)]
pub struct Token {
    pub id: u128,
    pub url: String,
    pub name: String,
    pub desc: String,
    // pub owner: Principal,
    pub properties: Vec<Property>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TokenDesc {
    pub id: u128,
    pub url: String,
    pub name: String,
    pub desc: String,
    pub owner: Principal,
    pub properties: Vec<Property>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TokenDescExt {
    pub id: u128,
    pub url: String,
    pub name: String,
    pub desc: String,
    pub owner: Principal,
    pub price: Option<u64>,
    pub properties: Vec<Property>,
}

#[derive(Serialize, CandidType, Deserialize, Default)]
pub struct State {
    pub owner: Option<Principal>,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub icon_url: String,

    pub total_supply: u32, //stores current number of minted tokens
    pub max_supply: u32, //maximum amount of tokens in collection

    // pub storage_canister: Option<Principal>,
    // pub ledger_canister: Option<Principal>,

    // pub creators_fee: u128,
    // pub creators_address: Principal,
    
    pub tokens: Vec<Token>, //Stores token data, this should not change
    pub token_owners: HashMap<u32, Principal>,//Stores token ownership
    // pub tx_enabled: bool,
    // pub token_listings: HashMap<u32, Listing>,

    //List of owners, with list of tokens
    pub owners: HashMap<Principal, Vec<u128>>,
    
    // pub assets: HashMap<String, (Vec<HeaderField>, Vec<u8>)>,

    //sorted by price from high to low, insertion involves finding correct place to insert
    // pub listed: Vec<Listing>
}

impl State {
    pub fn get() -> Rc<RefCell<State>> {
        STATE.with(|x| x.clone())
    }

    fn assign_to(&mut self, to: Principal, token_id: u32) {
        let list = self.owners.get_mut(&to);

        match list {
            Some(list) => {
                list.push(token_id as u128);
            }
            None => {
                let list = vec![token_id as u128];
                self.owners.insert(to, list);
            }
        }
    }

    fn remove_from(&mut self, from: Principal, token_id: u32) {
        let list = self.owners.get_mut(&from);

        match list {
            Some(list) => {
                let pos = list.iter().position(|&n| n == token_id as u128);
                if pos.is_some() {
                    list.remove(pos.unwrap());
                }
            }
            None => {
            }
        }
    }

    pub fn mint(&mut self, caller: Principal) -> Result<u32, String> {
        if self.token_owners.len() as u32 >= self.max_supply { return Err("Max token count reached".to_string()); }
    
        let token_id = self.total_supply+1;
        
        if self.token_owners.contains_key(&token_id) { return Err("Could not mint token that is already taken".to_string()); }

        //Mint token
        self.token_owners.insert(token_id, caller);

        //Add minted token to owner
        self.assign_to(caller, token_id);

        //Increase number of minted tokens
        self.total_supply += 1;

        LEDGER.with(|x| x.borrow_mut().mint(caller, caller, token_id));

        Ok(token_id)
    }

    pub fn mint_token_id(&mut self, caller: Principal, token_id: u32) -> Result<u32, String> {
        if self.token_owners.len() as u32 >= self.max_supply { return Err("Max token count reached".to_string()); }
    
        if self.token_owners.contains_key(&token_id) { return Err("Could not mint token that is already taken".to_string()); }

        //Mint token
        self.token_owners.insert(token_id, caller);

        //Add minted token to owner
        self.assign_to(caller, token_id);

        //Increase number of minted tokens
        self.total_supply += 1;

        LEDGER.with(|x| x.borrow_mut().mint(caller, caller, token_id));

        Ok(token_id)
    }
 
    pub fn check_token_id(&mut self,token_id: u32) -> Result<(), String> {
        if token_id > self.tokens.len() as u32 || token_id == 0  { return Err("Invalid token_id".to_string()); }

        Ok(())
    }

    pub fn get_owner(&mut self, token_id: u32) -> Result<Principal, String> {
        let owner = self.token_owners.get(&token_id).ok_or_else(|| String::from("Token not minted"))?;

        Ok(owner.clone())
    }

    pub fn check_owner(&mut self, token_id: u32, prin: Principal) -> Result<Principal, String> {
        let owner = self.token_owners.get(&token_id).ok_or_else(|| String::from("Token not minted"))?;

        //Check if current owner of the token is initiating transfer
        if *owner != prin {
            return Err(String::from("This token does not belong to caller"));
        }

        Ok(owner.clone())
    }

    pub fn moved(&mut self, from: Principal, to: Principal, token_id: u32) {
        //Change the owner of token_id
        self.token_owners.insert(token_id, to);

        //Update owner table
        self.remove_from(from, token_id);
        self.assign_to(to, token_id);
    }

    //Transfers token between accounts
    pub fn transfer(&mut self, from: Principal, to: Principal, token_id: u32) -> Result<u64, String> {
        //Check if token_id is between 0 and max_supply
        self.check_token_id(token_id)?;

        //Check if token was minted and sender is the owner of token_id
        self.check_owner(token_id, from)?;

        //Update owner table
        self.moved(from, to, token_id);

        MARKETPLACE.with(|x| x.borrow_mut().delist(from, token_id));

        //Update LEDGER
        let block = LEDGER.with(|x| x.borrow_mut().transfer(from, to, token_id));

        return Ok(block);
    }
} 

#[cfg(test)]
mod test {
use super::*;

    fn get_state() -> State {
        let owner = Principal::from_text("ucoje-n5scm-5ag2l-xpy42-o56he-nu5jr-iq3vm-25e7q-tuq5y-i7vpi-qae").unwrap();

        State {
            owner: Some(owner),
            name: String::from("name"),
            symbol: String::from("symbol"),
            description: String::from("description"),
            icon_url: "None".to_string(),
    
            max_supply: 10000 as u32,
            total_supply: 0,
    
            tokens: Vec::with_capacity(10000 as usize),
            token_owners: HashMap::default(),
            owners: HashMap::default()
        }
    }

    #[test]
    fn test_mint() {
        let mut state = get_state();
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();

        let mint_result = state.mint(prin);

        assert_eq!(mint_result, Ok(1));

        let len =  LEDGER.with(|x| x.borrow().tx.len());
        assert_eq!(len, 1);
    }

    #[test]
    fn test_mint_id() {
        let mut state = get_state();
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();

        let mint_result = state.mint_token_id(prin, 100);

        assert_eq!(mint_result, Ok(100));

        let len =  LEDGER.with(|x| x.borrow().tx.len());
        assert_eq!(len, 1);
    }
}