use std::rc::Rc;
use common::{ SendArgs, TransactionNotification, ICPTs };
use std::cell::RefCell;
use std::collections::HashMap;

use ic_cdk::api::time;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal, Decode};

use crate::token::STATE;
use crate::ledger::LEDGER;

use serde::Serialize;


thread_local! {
    pub static MARKETPLACE: Rc<RefCell<Marketplace>> = Rc::new(RefCell::new(Marketplace::default()));
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct Listing {
    pub owner: Principal,
    pub token_id: u32,
    pub price: u64,

    pub time: u64,
}

#[derive(Serialize, CandidType, Deserialize, Default)]
pub struct Marketplace {
    pub creators_fee: u128,
    pub creators_address: Option<Principal>,

    //Used to verify who is sending notification
    pub notify_canister: Option<Principal>,

    //Ledger canister principal
    pub ledger_canister: Option<Principal>,

    pub tx_enabled: bool,
    pub listings: HashMap<u32, Listing>,

    pub transfers: Vec<SendArgs>,
}

impl Marketplace {
    pub fn get() -> Rc<RefCell<Marketplace>> {
        MARKETPLACE.with(|x| x.clone())
    }

    fn is_tx_enabled(&mut self) -> Result<(), String> {
        if !self.tx_enabled { return Err(String::from("Transactions are not enabled")); }
        Ok(())
    }

    //Adds token to listing
    pub fn list(&mut self, from: Principal, token_id: u32, price: u64) -> Result<u64, String> {
        self.is_tx_enabled()?;

        STATE.with(|x| x.borrow_mut().check_token_id(token_id))?;

        //Check if current owner of the token is listing
        let owner = STATE.with(|x| x.borrow_mut().check_owner(token_id, from))?;

        if price < 1000000 { return Err(String::from("Minimum listing price is 0.01")); }

        //Get or update listing
        let listing = self.listings.get_mut(&token_id);

        match listing {
            Some(listing) => {
                listing.price = price;
            },
            None => {
                let item = Listing {
                    owner: owner,
                    token_id: token_id,
                    price: price,
                    time: time()
                };
                self.listings.insert(token_id, item);
            } 
        }

        //Add listing to ledger
        let block = LEDGER.with(|x| x.borrow_mut().list(from, token_id, price));

        return Ok(block);
    }

    //Removes token from listing
    pub fn delist(&mut self, from: Principal, token_id: u32) -> Result<u64, String>  {
        //Check if current owner of the token is listing
        let owner = STATE.with(|x| x.borrow_mut().check_owner(token_id, from))?;
        
        //Remove listing
        self.listings.remove(&token_id).ok_or_else(|| String::from("Token is not listed"))?;

        //Add delist to ledger
        let block = LEDGER.with(|x| x.borrow_mut().delist(from, token_id));

        Ok(block)
    }

    fn get_ledger_canister(&mut self) -> Result<Principal, String> {
        self.ledger_canister.ok_or_else(|| String::from("Ledger canister not set"))
    }

    // //Sends ICP to arbitrary principal id
    pub async fn send_icp(&mut self, to: Principal, amount: u64, memo: u64) -> Result<u64, String> {
        let ledger_canister = self.get_ledger_canister()?;

        let to_account = common::account_id(to, None).clone();
        
        let args = SendArgs {
            memo: memo,
            amount: ICPTs { e8s: amount },
            fee: ICPTs { e8s: 10000 },
            from_subaccount: None,
            to: to_account,
            created_at_time: None
        };

        //Encode args in candid
        let event_raw = ic_cdk::export::candid::encode_args(
            (args,)
        ).map_err(|_| String::from("Cannot serialize Transaction Args"))?;

        //Inter container call to ledger canister
        let raw_res = ic_cdk::api::call::call_raw(
            ledger_canister,
            "send_dfx",
            event_raw.clone(),
            0,
        ).await.map_err(|(_,s)| format!("Error invoking Ledger Canister, {}", &s))?;

        //Todo: deserialize send_dfx result to get block height!
        let res = Decode!(&raw_res, u64).map_err(|_| String::from("Error decoding response from Ledger canister"))?;

        Ok(res)
    }

    //Wrap for purchase, if failed returns funds to original caller 
    pub async fn purchase(&mut self, caller: Principal, args: TransactionNotification)-> Result<u64, String> {
        let result = self._purchase(caller, args.clone()).await;

        match result {
            Ok(_) => {}, //Everythin is fine do nothing
            Err(_) => { //Transaction error, return funds if possible

                if args.amount.e8s > 10000 { //Return funds only if the sent amount is enough
                    let _res = self.send_icp(args.from, args.amount.e8s-10000, args.memo).await;
                }
            }
        }

        return result;
    }

    async fn _purchase(&mut self, caller: Principal, args: TransactionNotification)-> Result<u64, String> {
        self.is_tx_enabled()?;
        
        let ledger_canister = self.get_ledger_canister()?;
        
        if caller != ledger_canister { return Err(String::from("Only ledger canister can call notify"));}

        let token_id = args.memo as u32;

        let listing = self.listings.get(&token_id).ok_or_else(|| String::from("Token is not listed"))?;


        // if token_id > self.tokens.len() as u128 || token_id == 0  { return Err("Invalid token_id"); }

        // let listing_pos = self.listed.iter().position(|x| x.token_id == token_id);
        // if listing_pos.is_none() { return Err("Token is not listed"); }

        // let listing = self.listed.get(listing_pos.unwrap()).unwrap().clone();
        if listing.price > args.amount.e8s { return Err(String::from("Sent amount does not satisfy listing price"));}

        
        //Remove listed position from listings, it was just purchased
        // self.listings.remove(&token_id);

        //Transfer token to purchaser
        // let pos = (token_id-1) as usize;
        // let token = self.tokens.get_mut(pos).unwrap();

        // token.owner = args.from;

        STATE.with(|x| x.borrow_mut().moved(listing.owner, args.from, token_id));
        // self.remove_from(listing.owner, token_id);
        // self.assign_to(args.from, token_id);

        //Calculate fee and amount to send
        let mut fee = (listing.price as u128 * self.creators_fee / 100000) as u64;
        let amount = listing.price - fee;

        //Include doble tx fees one for sending tokens to seller and one for sending tokens to creator
        fee = fee - 10000 - 10000;

        // match _token_result {
        //     Ok(_) => {},
        //     Err(_) => {
        //         return Err("Could not process transaction");
        //     }
        // }
        
        //Start await part, canister state can change during awaits

        // let _res = self.store_tx(caller, Operation::purchase, listing.owner, Some(args.from), token_id, Some(listing.price), time() as i128).await;

        //Send ICP to seller
        // let _token_result = self.send_icp(listing.owner, amount, token_id as u64).await;
        //Send Fee
        // let _fee_result = self.send_icp(self.creators_address, fee, token_id as u64).await;

        //Return surplus amount to sender
        if listing.price < args.amount.e8s {
            let surplus = args.amount.e8s - listing.price;

            if surplus > 10000 { //Return only if the surplus amount is bigger then tx fee 
                // let _fee_result = self.send_icp(args.from, surplus-10000, token_id as u64).await;
            }
        }

        let block = LEDGER.with(|x| x.borrow_mut().purchase(caller, listing.owner, args.from, token_id, listing.price));

        return Ok(block);
    }
}