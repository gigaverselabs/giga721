use serde_bytes::ByteBuf;
use common::rc_bytes::RcBytes;
use crate::storage::Asset;
use std::cell::RefCell;
use std::io;
use std::collections::HashMap;

use ic_cdk::api::stable::StableMemoryError;

use ic_cdk::export::candid::{Principal};

use crate::token::State;
use crate::marketplace::{Marketplace, Stats};

use common::SendArgs;
use ic_cdk::export::candid::{encode_args};

pub fn trap(data: &str) {
    panic!("{}",data);
}

pub async fn call_send_dfx(_canister: Principal, args: &SendArgs) -> Result<u64, String> {
    //Encode args in candid
    let _event_raw = encode_args((args,))
        .map_err(|_| String::from("Cannot serialize Transaction Args"))?;
        
    Ok(0)
}

pub fn get_state() -> State {
    let owner = user_a();

    State {
        owner: Some(owner),
        name: String::from("name"),
        symbol: String::from("symbol"),
        description: String::from("description"),
        icon_url: "None".to_string(),

        max_supply: 10000 as u32,
        total_supply: 0,

        is_paused: false,

        tokens: HashMap::default(),
        token_owners: HashMap::default(),
        owners: HashMap::default()
    }
}

pub fn set_state() {
    let state = get_state();
    *State::get().borrow_mut() = state;
}


pub fn get_marketplace() -> Marketplace {
    let owner = user_a();
    let ledger = ledger();

    Marketplace {
        creators_fee: 2500,
        creators_address: Some(owner),

        notify_canister: Some(ledger),

        ledger_canister: Some(ledger),

        tx_enabled: true,
        listings: HashMap::default(),

        payment_offset: 0,
        listing_offset: 0,
        payments: Vec::default(),
        stats: Stats::default()
    }
}

pub fn set_marketplace() {
    let state = get_marketplace();
    *Marketplace::get().borrow_mut() = state;
}

pub fn user_a() -> Principal {
    Principal::from_text("ucoje-n5scm-5ag2l-xpy42-o56he-nu5jr-iq3vm-25e7q-tuq5y-i7vpi-qae").unwrap()
}

pub fn user_b() -> Principal {
    Principal::from_text("mjfyj-22dca-dcahz-umwwq-vpe4r-iukdj-uuymz-fvphz-rt6my-g7vrs-5qe").unwrap()
}

pub fn ledger() -> Principal {
    Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()
}

pub fn get_asset() -> Asset {
    Asset {
        name: "example asset".to_string(),
        content_type: "image/jpg".to_string(),
        data: RcBytes::from(ByteBuf::from(vec![243; 1024*1024])),
    }
}

thread_local! {
    static STORAGE: RefCell<Vec<u8>> = RefCell::new(vec![]);
}

// pub fn clear_storage() {
//     STORAGE.with(|s| {
//         s.borrow_mut().clear();
//     });
// }

pub fn time() -> u64 {
    0
}

/// Return the page count, not the total bytes in storage.
/// This is how ic_cdk works
pub fn stable_size() -> u32 {
    STORAGE.with(|s| s.borrow().len()) as u32 >> 16
}

// pub fn stable_bytes() -> Vec<u8> {
//     let size = (stable_size() as usize) << 16;
//     let mut vec = Vec::with_capacity(size);

//     // This is super dodgy, don't do this.
//     // This is copied from the current implementation of stable storage.
//     #[allow(clippy::uninit_vec)]
//     unsafe {
//         vec.set_len(size);
//     }

//     stable_read(0, vec.as_mut_slice());

//     vec
// }

pub fn stable_read(offset: u32, buf: &mut [u8]) {
    STORAGE.with(|storage| {
        let offset = offset as usize;
        buf.copy_from_slice(&storage.borrow()[offset..offset + buf.len()]);
    });
}

pub fn stable_write(offset: u32, buf: &[u8]) {
    STORAGE.with(|storage| {
        let offset = offset as usize;
        storage.borrow_mut()[offset..offset + buf.len()].copy_from_slice(buf);
    });
}

pub fn stable_grow(new_pages: u32) -> Result<u32, StableMemoryError> {
    STORAGE.with(|storage| {
        let additional_len = (new_pages << 16) as usize;
        let len = storage.borrow().len();
        match len + additional_len >= u32::MAX as usize {
            false => {
                let previous_size = storage.borrow().len() >> 16;
                storage.borrow_mut().append(&mut vec![0u8; additional_len]);
                Ok(previous_size as u32)
            }
            true => Err(StableMemoryError()),
        }
    })
}

/// A writer to the stable memory.
///
/// Will attempt to grow the memory as it writes,
/// and keep offsets and total capacity.
pub struct StableWriter {
    /// The offset of the next write.
    offset: usize,

    /// The capacity, in pages.
    capacity: u32,
}

impl Default for StableWriter {
    fn default() -> Self {
        let capacity = stable_size();

        Self {
            offset: 0,
            capacity,
        }
    }
}

impl StableWriter {
    /// Attempts to grow the memory by adding new pages.
    pub fn grow(&mut self, added_pages: u32) -> Result<(), StableMemoryError> {
        let old_page_count = stable_grow(added_pages)?;
        self.capacity = old_page_count + added_pages;
        Ok(())
    }

    /// Writes a byte slice to the buffer.
    ///
    /// The only condition where this will
    /// error out is if it cannot grow the memory.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
        if self.offset + buf.len() > ((self.capacity as usize) << 16) {
            self.grow((buf.len() >> 16) as u32 + 1)?;
        }

        stable_write(self.offset as u32, buf);
        self.offset += buf.len();
        Ok(buf.len())
    }
}

impl io::Write for StableWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.write(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Out Of Memory"))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // Noop.
        Ok(())
    }
}

/// A reader to the stable memory. Keeps an offset and reads off stable memory
/// consecutively.
pub struct StableReader {
    /// The offset of the next write.
    offset: usize,
}

impl Default for StableReader {
    fn default() -> Self {
        Self { offset: 0 }
    }
}


impl StableReader {
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
        stable_read(self.offset as u32, buf);
        self.offset += buf.len();
        Ok(buf.len())
    }

    // pub fn set_offset(&mut self, offset: usize) {
    //     self.offset = offset;
    // }
}

impl io::Read for StableReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unexpected error."))
    }
}