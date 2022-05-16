use crate::int_set::IntSet;
use std::cell::RefCell;
use std::rc::Rc;

use ic_cdk::api::call::call_raw;
use ic_cdk::api::time;
use ic_cdk::export::candid::{CandidType, Decode, Deserialize, Principal};
use ic_cdk::{caller, id};

use prost::Message;

use common::account_identifier::AccountIdentifier;
use common::{call_send_dfx, ICPTs, SendArgs, TransactionNotification, TransactionResponse};

thread_local! {
    pub static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State::default()));
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct NotificationStatus {
    pub index: u64,
    pub timestamp: u64,
    pub args: TransactionNotification,
    pub result: Option<Result<TransactionResponse, AppErr>>,
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct TransferStatus {
    pub index: u64,
    pub timestamp: u64,
    pub args: SendArgs,
    pub result: Option<Result<u64, String>>,
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct ProxyStatus {
    pub total_market_fee: u64,
    pub total_creator_fee: u64,

    pub waiting_market_fee: u64,
    pub waiting_creator_fee: u64,
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, Serialize)]
pub enum ErrType {
    Call,
    Decode,
    Resp,
    Token,
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct AppErr {
    id: ErrType,
    text: String,
}

use serde::Serialize;

#[derive(Clone, Deserialize, Serialize)]
pub struct State {
    pub ledger_canister: Option<Principal>,
    pub token_canister: Option<Principal>,

    pub owner: Option<Principal>,
    pub blocks: Vec<SendArgs>,
    pub blocks_processed: IntSet,

    pub market_fee: u64,
    pub market_address: Option<Principal>,

    pub total_market_fee: u64,
    pub total_creator_fee: u64,

    pub waiting_market_fee: u64,
    pub waiting_creator_fee: u64,

    ///Stores all ICP transfers with results (success or error)
    pub payment_offset: u64,
    pub payments: Vec<TransferStatus>,

    pub notification_offset: u64,
    pub notifications: Vec<NotificationStatus>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ledger_canister: Some(Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()),
            token_canister: None,

            owner: None,
            blocks: Vec::default(),
            blocks_processed: IntSet::default(),
            // blocks_processed: IntMap::new(),

            market_fee: 2500,
            market_address: None,

            total_market_fee: 0,
            total_creator_fee: 0,

            waiting_market_fee: 0,
            waiting_creator_fee: 0,

            payment_offset: 0,
            payments: Vec::default(),

            notification_offset: 0,
            notifications: Vec::default(),
        }
    }
}

use crate::ledger;

impl State {
    ///Checks if block was already processed, returns Ok if block is not processed, err if processed
    pub fn is_processed(&self, block: u64) -> Result<(), String> {
        let block = self.blocks_processed
            .get(block);

        match block {
            Some(_) => {
                Err(String::from("Block already processed"))
            }
            None => {
                Ok(())
            }
        }
    }

    ///Downloads block from ledger and send notification to token canister
    pub async fn notify(&mut self, block: u64) -> Result<(), String> {
        let caller = caller();
        let id = id();

        //Preconditions block must not be processed, token_canister must be set
        self.is_processed(block)?;
        let token_canister = self
            .token_canister
            .ok_or_else(|| String::from("Token canister not set!"))?;
        //Add block to processed list
        self.blocks_processed.insert(block, ());

        //Get send transaction data from ICP ledger archive
        let (_from, _to, amount, memo) = self.get_send_transaction(block).await?;


        //Verify that transaction is for our canister, otherwise throw error
        let caller_account_id = AccountIdentifier::new(caller.clone(), None);
        let canister_account_id = AccountIdentifier::new(id.clone(), None);

        //Check if receiving canister is current one
        if canister_account_id != _to {
            return Err(String::from(
                "Invalid block! Canister does not match block recipient",
            ));
        }

        //From this point if there is an error in processing payment, payment should be returned to sender
        let result = {
            if caller_account_id != _from {
                Err(AppErr {
                    id: ErrType::Resp,
                    text: String::from("Invalid block! Caller does not match block sender"),
                })
            } else {
                //Send notification to token canister
                self.notify_canister(caller.clone(), token_canister, block, amount, memo)
                    .await
            }
        };

        match result {
            Ok(res) => {
                //Process response from the token canister, sends ICP to seller
                self.process_response(amount, &res).await?;

                Ok(())
            }
            Err(s) => {
                if s.id != ErrType::Call {
                    self.refund(_from.to_hex(), amount, memo).await;
                }

                Err(s.text)
            }
        }

        // return result;
    }

    ///Processes response from token canister, if it is success send ICP to seller and store fees for disbursment
    async fn process_response(
        &mut self,
        amount: u64,
        resp: &TransactionResponse,
    ) -> Result<(), String> {
        let market_fee = self.market_fee * amount / 100000;
        let creators_fee = resp.creators_fee * amount / 100000;

        let seller = amount - market_fee - creators_fee;

        self.waiting_market_fee += market_fee;
        self.total_market_fee += market_fee;

        self.waiting_creator_fee += creators_fee;
        self.total_creator_fee += creators_fee;

        //Send remaining icp to seller
        let _block = self
            .send_icp(common::account_id(resp.seller, None), seller, 0)
            .await?;
        Ok(())
    }

    /// Returns funds back to the sender, works only if funds are bigger than TX_FEE
    async fn refund(&mut self, to_account: String, amount: u64, memo: u64) {
        if amount > common::TX_FEE {
            let _block = self
                .send_icp(to_account, amount - common::TX_FEE, memo)
                .await;
        }
    }

    fn get_ledger_canister(&self) -> Result<Principal, String> {
        self.ledger_canister
            .ok_or_else(|| String::from("Ledger canister not set"))
    }

    /// Send ICP to target principal, subaccount
    async fn send_icp(
        &mut self,
        to_account: String,
        amount: u64,
        memo: u64,
    ) -> Result<u64, String> {
        let ledger_canister = self.get_ledger_canister()?;

        // let to_account = common::account_id(to, None).clone();
        let args = SendArgs {
            memo: memo,
            amount: ICPTs { e8s: amount },
            fee: ICPTs {
                e8s: common::TX_FEE,
            },
            from_subaccount: None,
            to: to_account,
            created_at_time: None,
        };

        self.payment_offset += 1;
        let mut payment = TransferStatus {
            index: self.payment_offset,
            args: args.clone(),
            timestamp: time(),
            result: None,
        };

        let result = call_send_dfx(ledger_canister, &args).await;

        payment.result = Some(result.clone());

        self.payments.push(payment);

        return result;
    }

    async fn notify_canister(
        &mut self,
        caller: Principal,
        canister: Principal,
        block: u64,
        amount: u64,
        memo: u64,
    ) -> Result<TransactionResponse, AppErr> {
        let transaction_notification_args = TransactionNotification {
            from: caller,
            from_subaccount: None,
            to: canister,
            to_subaccount: None,
            block_height: block,
            amount: ICPTs { e8s: amount },
            memo: memo,
        };

        let res = {
            let event_raw =
                ic_cdk::export::candid::encode_args((transaction_notification_args.clone(),))
                    .unwrap();
            //Notify token canister
            let raw_res = call_raw(canister, "transaction_notification", event_raw.clone(), 0)
                .await
                .map_err(|(_, s)| AppErr {
                    id: ErrType::Call,
                    text: format!("Error while calling token canister, {}", s),
                })?;

            let res = Decode!(&raw_res, Result<TransactionResponse, String>).map_err(|_| AppErr {
                id: ErrType::Decode,
                text: format!("Error while decoding response"),
            })?.map_err(|s| AppErr {
                id: ErrType::Token,
                text: format!("Error from token canister, {}", s),
            });

            res
        };
        self.notification_offset += 1;
        self.notifications.push(NotificationStatus {
            index: self.notification_offset,
            timestamp: time(),
            args: transaction_notification_args,
            result: Some(res.clone()),
        });

        let res = res?;

        Ok(res)
    }

    ///Downloads block from ledger and returns parsed information, returns only send transactions
    async fn get_send_transaction(
        &self,
        block_height: u64,
    ) -> Result<(AccountIdentifier, AccountIdentifier, u64, u64), String> {
        //Get block from ledger
        let raw_block: ledger::EncodedBlock = self.get_block(block_height).await?;
        let block_data = ledger::Block::decode(&raw_block.block[..])
            .map_err(|x| format!("Could not decode block, {}", x))?;
        let transaction = block_data
            .transaction
            .ok_or_else(|| String::from("Transaction is None"))?;

        let (_from, _to, amount) = match transaction
            .transfer
            .ok_or_else(|| String::from("Transaction transfer is none"))?
        {
            ledger::transaction::Transfer::Send(item) => (
                AccountIdentifier::from_slice(&item.from.unwrap().hash[..]).unwrap(),
                AccountIdentifier::from_slice(&item.to.unwrap().hash[..]).unwrap(),
                item.amount
                    .ok_or_else(|| String::from("Amount is empty!"))?
                    .e8s,
            ),
            ledger::transaction::Transfer::Burn(_) => {
                return Err(String::from(
                    "Notification failed transfer must be of type send, found burn",
                ))
            }
            ledger::transaction::Transfer::Mint(_) => {
                return Err(String::from(
                    "Notification failed transfer must be of type send, found mint",
                ))
            }
        };

        Ok((
            _from,
            _to,
            amount,
            transaction.memo.map_or_else(|| 0, |s| s.memo),
        ))
    }

    async fn get_block(&self, block_height: u64) -> Result<ledger::EncodedBlock, String> {
        //ICP Ledger canister
        let canister = self.get_ledger_canister()?;

        let mut req = ledger::BlockRequest::default();
        req.block_height = block_height;
        let mut buf = Vec::<u8>::new();
        buf.reserve(req.encoded_len());
        req.encode(&mut buf).unwrap();

        let res = call_raw(canister, "block_pb", buf, 0)
            .await
            .map_err(|(_, text)| text)?;

        let resp = ledger::BlockResponse::decode(&res[..])
            .map_err(|x| format!("Prost decoding error {}", x))?;
        match resp.block_content.unwrap() {
            ledger::block_response::BlockContent::Block(raw_block) => Ok(raw_block),
            ledger::block_response::BlockContent::CanisterId(_prin) => {
                Err(String::from("Block is already archived!"))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_processed() {
        let mut state = State::default();

        let processed = state.is_processed(1234);
        
        assert_eq!(processed, Ok(()));
    }
}