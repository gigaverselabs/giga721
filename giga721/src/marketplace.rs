use std::rc::Rc;
use common::{ SendArgs, TransactionNotification, ICPTs };
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(test)]
use crate::testing::{time};
#[cfg(test)]
use crate::testing::{call_send_dfx};

#[cfg(not(test))]
use ic_cdk::api::time;
#[cfg(not(test))]
use common::call_send_dfx;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use crate::token::STATE;
use crate::ledger::LEDGER;

use serde::Serialize;

use ic_cdk_macros::{query};

thread_local! {
    pub static MARKETPLACE: Rc<RefCell<Marketplace>> = Rc::new(RefCell::new(Marketplace::default()));
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct Payment {
    pub index: u64,
    pub time: u64,
    pub args: SendArgs,
    // pub status: PaymentStatus,
    pub block_height: Option<u64>,
    pub error: Option<String>
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct Listing {
    pub index: u64,
    pub owner: Principal,
    pub token_id: u32,
    pub price: u64,

    pub time: u64,
}

#[derive(Serialize, CandidType, Deserialize, Default, Clone)]
pub struct Stats {
    pub highest_sell: u64,
    pub volume_traded: u64,
}

#[derive(Serialize, CandidType, Deserialize, Default, Clone)]
pub struct StatsResult {
    pub highest_sell: u64,
    pub volume_traded: u64,
    pub owners: u64,
    pub listings: u64
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
    
    pub listing_offset: u64,
    pub listings: HashMap<u32, Listing>,

    pub payment_offset: u64,
    pub payments: Vec<Payment>,

    pub stats: Stats,
}

#[query]
pub fn stats() -> StatsResult {
    let stats = MARKETPLACE.with(|x| { x.borrow().stats.clone() });
    StatsResult {
        highest_sell: stats.highest_sell,
        volume_traded: stats.volume_traded,
        owners: STATE.with(|x| { x.borrow().owners.len() }) as u64,
        listings: MARKETPLACE.with(|x| x.borrow().listings.len() ) as u64,
    }
}

impl Marketplace {
    pub fn get() -> Rc<RefCell<Marketplace>> {
        MARKETPLACE.with(|x| x.clone())
    }

    fn is_tx_enabled(&mut self) -> Result<(), String> {
        if !self.tx_enabled { return Err(String::from("Transactions are not enabled")); }
        Ok(())
    }

    ///Adds token to listing
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
                self.listing_offset += 1;
                let item = Listing {
                    index: self.listing_offset,
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

    ///Removes token from listing
    pub fn delist(&mut self, from: Principal, token_id: u32) -> Result<u64, String>  {
        //Check if current owner of the token is listing
        STATE.with(|x| x.borrow_mut().check_owner(token_id, from))?;
        
        //Remove listing
        self.listings.remove(&token_id).ok_or_else(|| String::from("Token is not listed"))?;

        //Add delist to ledger
        let block = LEDGER.with(|x| x.borrow_mut().delist(from, token_id));

        Ok(block)
    }

    fn get_ledger_canister(&mut self) -> Result<Principal, String> {
        self.ledger_canister.ok_or_else(|| String::from("Ledger canister not set"))
    }

    ///Sends ICP to arbitrary principal id
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

        self.payment_offset += 1;
        let payments = Payment {
            index: self.payment_offset,
            args: args.clone(),
            time: time(),
            block_height: None,
            error: None
        };

        self.payments.push(payments);

        let block_height = call_send_dfx(ledger_canister, &args).await?;

        Ok(block_height)
    }

    //Wrap for purchase, if failed returns funds to original caller 
    pub async fn purchase(&mut self, caller: Principal, args: &TransactionNotification)-> Result<u64, String> {
        let result = self._purchase(caller, args).await;

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

    fn update_stats(&mut self, price: u64) {
        //Update stats
        self.stats.volume_traded += price;
        if price > self.stats.highest_sell {
            self.stats.highest_sell = price;
        }
    }

    async fn _purchase(&mut self, caller: Principal, args: &TransactionNotification)-> Result<u64, String> {
        self.is_tx_enabled()?;
        let ledger_canister = self.get_ledger_canister()?;
        //Check if ledger canister is sending notification
        if caller != ledger_canister { return Err(String::from("Only ledger canister can call notify"));}
        let token_id = args.memo as u32;
        //Check if token is listed
        let listing = self.listings.get(&token_id).ok_or_else(|| String::from("Token is not listed"))?.clone();
        //Check if amount is enough for listing
        if listing.price > args.amount.e8s { return Err(String::from("Sent amount does not satisfy listing price"));}

        //Remove listed position from listings, it was just purchased
        self.listings.remove(&token_id);

        //Move token from seller to buyer
        STATE.with(|x| x.borrow_mut().moved(listing.owner, args.from, token_id));

        //Add purchase to ledger
        let block = LEDGER.with(|x| x.borrow_mut().purchase(caller, listing.owner, args.from, token_id, listing.price));

        //Update stats
        self.update_stats(listing.price);

        //Calculate fee and amount to send
        let mut fee = (listing.price as u128 * self.creators_fee / 100000) as u64;
        let amount = listing.price - fee;

        //Include doble tx fees one for sending tokens to seller and one for sending tokens to creator
        fee = fee - 10000 - 10000;

        //Send ICP to seller
        let _token_result = self.send_icp(listing.owner, amount, token_id as u64).await?;
        //Send Fee, TODO: this can be postponed, less items to call
        let _fee_result = self.send_icp(self.creators_address.unwrap(), fee, token_id as u64).await?;

        //Return surplus amount to sender
        if listing.price < args.amount.e8s {
            let surplus = args.amount.e8s - listing.price;

            if surplus > 10000 { //Return only if the surplus amount is bigger then tx fee 
                let _fee_result = self.send_icp(args.from, surplus-10000, token_id as u64).await;
            }
        }

        return Ok(block);
    }
}

#[cfg(test)] 
mod test {
use super::*;

use crate::testing::*;

    #[test]
    fn test_list() {
        set_state();

        let owner = user_a();
        let mint_result = STATE.with(|x| x.borrow_mut().mint_token_id(owner, owner, 1));

        assert_eq!(mint_result, Ok(1));

        Marketplace::get().borrow_mut().tx_enabled = true;

        let list = Marketplace::get().borrow_mut().list(user_a(), 1, 100000000);

        assert_eq!(list, Ok(0));
    }

    #[test]
    fn test_list_delist() {
        set_state();

        let owner = user_a();
        let mint_result = STATE.with(|x| x.borrow_mut().mint_token_id(owner, owner, 1));

        assert_eq!(mint_result, Ok(0));

        Marketplace::get().borrow_mut().tx_enabled = true;

        let list = Marketplace::get().borrow_mut().list(user_a(), 1, 100000000);
        assert_eq!(list, Ok(1));

        let list = Marketplace::get().borrow_mut().delist(user_a(), 1);
        assert_eq!(list, Ok(2));
    }


    #[tokio::test]
    async fn test_purchase() {
        set_state();
        set_marketplace();

        let owner = user_a();
        let mint_result = STATE.with(|x| x.borrow_mut().mint_token_id(owner, owner, 1));

        assert_eq!(mint_result, Ok(0));

        Marketplace::get().borrow_mut().tx_enabled = true;

        let list = Marketplace::get().borrow_mut().list(user_a(), 1, 1000000);
        assert_eq!(list, Ok(1));

        let args = TransactionNotification {
            amount: ICPTs {
                e8s: 1000000
            },
            block_height: 12345,
            from: owner,
            from_subaccount: None,
            memo: 1,
            to: user_b(),
            to_subaccount: None
        };

        let purchase = Marketplace::get().borrow_mut().purchase(ledger(), &args).await;
        assert_eq!(purchase, Ok(2));

        let payments = Marketplace::get().borrow().payments.clone();

        assert_eq!(payments.len(), 2);
    }
}