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
    pub owner: Option<Principal>,
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

    pub is_paused: bool, //pauses any changes to contract, used for upgrade procedure

    /// Stores current number of minted tokens
    pub total_supply: u32, 
    pub max_supply: u32, //maximum amount of tokens in collection

    /// Stores token data, this should not change, this contains only token metadata, not actual minted tokens
    pub tokens: HashMap<u32, Token>, 
    /// Stores token ownership, this contains minted tokens
    pub token_owners: HashMap<u32, Principal>,

    /// List of owners, with list of tokens
    pub owners: HashMap<Principal, Vec<u128>>,
}

impl State {
    pub fn get() -> Rc<RefCell<State>> {
        STATE.with(|x| x.clone())
    }

    //Stores token metadata
    pub fn store_tokens(&mut self, tokens: &Vec<Token>) {
        for i in tokens {
            self.tokens.insert(i.id as u32, i.clone());
        }
    }

    pub fn data_of(&mut self, token_id: u32) -> Result<TokenDesc, String> {
        self.check_token_id(token_id)?;

        let data = self.tokens.get(&token_id).ok_or_else(|| String::from("Could not find token"))?;

        let item = TokenDesc {
            id: data.id,
            url: data.url.clone(),
            name: data.name.clone(),
            desc: data.desc.clone(),
            properties: data.properties.clone(),
            owner: self.get_owner(token_id).unwrap()
        };

        Ok(item)
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

    /// Removes token_id from owner helpers list
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

    /// Mints new token. Returns id of minted_token
    #[allow(dead_code)]
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

    pub fn mint_token_id(&mut self, caller: Principal, to: Principal, token_id: u32) -> Result<u64, String> {
        if self.token_owners.len() as u32 >= self.max_supply { return Err("Max token count reached".to_string()); }
    
        if token_id < 0 as u32 || token_id > self.max_supply { return Err("Token id outside of estabished bounds".to_string())}

        if self.token_owners.contains_key(&token_id) { return Err("Could not mint token that is already taken".to_string()); }

        //Mint token
        self.token_owners.insert(token_id, to);

        //Add minted token to owner
        self.assign_to(to, token_id);

        //Increase number of minted tokens
        self.total_supply += 1;

        let block_id = LEDGER.with(|x| x.borrow_mut().mint(caller, caller, token_id));

        Ok(block_id)
    }

    /// Burns token with @token_id, it can be executed only by token owner, Returns id of burned token
    pub fn burn(&mut self, caller: Principal, token_id: u32) -> Result<u64, String> {
        self.check_owner(token_id, caller)?; //Check if caller is the owner of token_id

        //Burn token
        self.token_owners.remove(&token_id).ok_or_else(|| "Canot remove token owner".to_string())?;

        self.remove_from(caller, token_id);

        //Decrease number of minted tokens
        self.total_supply -= 1;

        let block_id = LEDGER.with(|x| x.borrow_mut().burn(caller, caller, token_id));

        Ok(block_id)
    }
 
    /// Checks if token with given id was minted
    pub fn check_token_id(&mut self,token_id: u32) -> Result<(), String> {
        if !self.token_owners.contains_key(&token_id) {
            return Err("Invalid token_id".to_string())
        }
        // if token_id > self.tokens.len() as u32 || token_id == 0  { return Err("Invalid token_id".to_string()); }

        Ok(())
    }

    /// Returns the owner of given token_id or Err if token is not minted
    pub fn get_owner(&mut self, token_id: u32) -> Result<Principal, String> {
        let owner = self.token_owners.get(&token_id).ok_or_else(|| String::from("Token not minted"))?;

        Ok(owner.clone())
    }

    /// Verifies that the owner of given token_id is @prin
    pub fn check_owner(&mut self, token_id: u32, prin: Principal) -> Result<Principal, String> {
        let owner = self.token_owners.get(&token_id).ok_or_else(|| String::from("Token not minted"))?;

        //Check if current owner of the token is initiating transfer
        if *owner != prin {
            return Err(String::from("This token does not belong to caller"));
        }

        Ok(owner.clone())
    }

    /// Updated owners info and owner lookup table
    pub fn moved(&mut self, from: Principal, to: Principal, token_id: u32) {
        //Change the owner of token_id
        self.token_owners.insert(token_id, to);

        //Update owner table
        self.remove_from(from, token_id);
        self.assign_to(to, token_id);
    }

    /// Transfers token between accounts
    pub fn transfer(&mut self, from: Principal, to: Principal, token_id: u32) -> Result<u64, String> {
        //Check if token_id is between 0 and max_supply
        self.check_token_id(token_id)?;

        //Check if token was minted and sender is the owner of token_id
        self.check_owner(token_id, from)?;

        //First take off listing
        let _ = MARKETPLACE.with(|x| x.borrow_mut().delist(from, token_id));

        //Update owner table
        self.moved(from, to, token_id);

        //Update LEDGER
        let block = LEDGER.with(|x| x.borrow_mut().transfer(from, to, token_id));

        return Ok(block);
    }
} 

#[cfg(test)]
mod test {
use super::*;
use crate::testing::*;


    #[test]
    fn test_transfer() {
        let mut state = get_state();
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();
        let to = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();
        let token_id = 1 as u32;

        let mint_result = state.mint(prin);

        assert_eq!(mint_result, Ok(1));

        let len =  LEDGER.with(|x| x.borrow().tx.len());
        assert_eq!(len, 1);
        
        state.transfer(prin, to, token_id).unwrap();
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
    fn test_burn() {
        let mut state = get_state();
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();

        let mint_result = state.mint(prin);
        let minted_id = mint_result.unwrap();
        
        assert_eq!(minted_id, 1);

        let len =  LEDGER.with(|x| x.borrow().tx.len());
        assert_eq!(len, 1);

        let burned_id = state.burn(prin, minted_id);
        // assert_eq!(burned_id.unwrap(), minted_id);

        let len2 =  LEDGER.with(|x| x.borrow().tx.len());
        assert_eq!(len2, 2);
    }

    #[test]
    fn test_mint_id() {
        let mut state = get_state();
        let prin = Principal::from_text("tushn-jfas4-lrw4y-d3hun-lyc2x-hr2o2-2spfo-ak45s-jzksj-fzvln-yqe").unwrap();

        let mint_result = state.mint_token_id(prin, prin, 100);

        assert_eq!(mint_result, Ok(100));

        let len =  LEDGER.with(|x| x.borrow().tx.len());
        assert_eq!(len, 1);
    }
}