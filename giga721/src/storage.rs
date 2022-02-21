use std::rc::Rc;
use common::HeaderField;
use std::collections::HashMap;
use ic_cdk::api::stable::StableMemoryError;

use ic_cdk::export::candid::{CandidType};
use serde_cbor::{to_vec, from_slice};

use serde::{Serialize, Deserialize};

use std::cell::RefCell;

#[cfg(test)]
use crate::testing::{stable_read, stable_write, stable_size, stable_grow};

#[cfg(not(test))]
use ic_cdk::api::stable::{stable_read, stable_write, stable_size, stable_grow};

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct Asset {
    pub name: String,
    pub headers: Vec<HeaderField>,
    pub data: Vec<u8>
}

#[derive(Serialize, Deserialize, Default)]
pub struct StableStorage {
    pub assets: HashMap<String, (u32, u32, Vec<HeaderField>)>, //Stores offset and size of asset

    pub size: u32, //Stores size of all assets along with data pages

    pub state_offset: u32, //Keeps information about offset for state
    pub state_size: u32 //Stored state_size
}

thread_local! {
    pub static STORAGE: Rc<RefCell<StableStorage>> = Rc::new(RefCell::new(StableStorage::default()));
}

impl StableStorage {
    pub fn get() -> Rc<RefCell<StableStorage>> {
        STORAGE.with(|x| x.clone())
    }

    //Initialize stable storage data structure, use with caution, this will wipe all data in st able storage!
    pub fn init_storage(&mut self) -> Result<(),()> {
        match self.grow(1) {
            Err(_) => {
                Err(())
            }
            Ok(_) => {
                stable_write(0, &[0]);
                self.size = 9;

                Ok(())
            }
        }//initialize stable storage if necessary
    }

    ///Ensures that there is enough space in stable memory
    fn grow(&mut self, size: u32) -> Result<(), StableMemoryError> {
        if self.size + size > stable_size() << 16 {
            stable_grow((size >> 16) +1)?;
        }

        Ok(())
    }

    //Stores asset in stable memory,
    //data structure: name_length, name_utf8, header_length, headers_cbor, data_length, data_array
    pub fn store_asset(&mut self, asset: &Asset) -> Result<(), String> {
        //Throw Err result if there is an error with serialization of data
        let vec = to_vec(&asset.headers).map_err(|err| format!("{}", err))?;

        //header size + data length info (u32) + actual data
        let length = (vec.len() + 4 + asset.name.as_bytes().len() + 4 + asset.data.len() + 4) as u32;
        
        //Make sure the storage is large enough, otherwise return Err
        self.grow(length).map_err(|_| String::from("Stable memory error"))?;
        
        //Write Asset Name length
        stable_write(self.size, &(asset.name.as_bytes().len() as u32).to_be_bytes());
        self.size+=4;
        //Write Asset name
        stable_write(self.size, &asset.name.as_bytes());
        self.size+=asset.name.as_bytes().len() as u32;

        //Write headers data length
        stable_write(self.size, &(vec.len() as u32).to_be_bytes());
        self.size+=4;
        
        //Write headers data
        stable_write(self.size, &vec);
        self.size+=vec.len() as u32;

        //Write data length
        stable_write(self.size, &(asset.data.len() as u32).to_be_bytes());
        self.size+=4;

        //Todo: requires fixing
        self.assets.insert(asset.name.clone(), (self.size, asset.data.len() as u32, asset.headers.clone()));

        //Write data
        stable_write(self.size, &asset.data);
        self.size+=asset.data.len() as u32;

        //Update number of stored assets
        stable_write(0, &[self.assets.len() as u8]);

        return Ok(());
    }

    pub fn get_asset(&mut self, name: &str) -> Result<(Vec<HeaderField>, Vec<u8>),String> {
        let (offset, size, headers) = self.assets.get(name).ok_or_else(|| format!("Asset not found {}", name))?;

        let mut buf = vec![0; *size as usize];

        stable_read(*offset, &mut buf);

        Ok((headers.clone(), buf))
    }

    //Load assets information from the stable storage, it does not load all asset data in to cache, only names and headers
    fn load_assets(&mut self) -> Result<(), String> {
        if stable_size() == 0 {
            return Err(String::from("No data in stable storage"));
        }

        //Clean AssetStorage
        self.assets.clear();

        //Load number of items to process, u8
        let mut items = [0];
        stable_read(0, &mut items);

        let mut raw_data = vec![0;self.size as usize];
        stable_read(0, &mut raw_data);

        //Skip state info
        let mut offset = 9;

        for i in 0 .. items[0] {
            //Read name of the asset
            let mut u32_buf : [u8;4] = [0,0,0,0]; 

            stable_read(offset, &mut u32_buf);
            offset += 4;

            let name_size = u32::from_be_bytes(u32_buf);

            let mut name_vec = vec![0; name_size as usize];

            stable_read(offset, &mut name_vec);
            offset += name_size;

            let name = String::from_utf8(name_vec).map_err(|_| String::from("Error on utf8 conversion of asset name"))?;

            //Read headers of assets
            stable_read(offset, &mut u32_buf);
            offset += 4;
            let headers_size = u32::from_be_bytes(u32_buf);

            let mut headers_vec = vec![0; headers_size as usize];
            stable_read(offset, &mut headers_vec);
            offset += headers_size;

            let headers : Vec<HeaderField> = from_slice(&headers_vec).map_err(|_| String::from("Could not parse data"))?;

            //Read data length
            stable_read(offset, &mut u32_buf);
            offset += 4;
            let data_size = u32::from_be_bytes(u32_buf);

            self.assets.insert(name.clone(), (offset, data_size, headers));

            // //test load data
            // let mut data = vec![0;data_size as usize];
            // stable_read(offset, &mut data);
        }
 
        return Ok(());
    }

    pub fn store_state<T>(&mut self, t: T) -> Result<(), String> 
        where
    // T: candid::CandidType,
    T: serde::Serialize,
    {
        let vec = to_vec(&t).map_err(|err| format!("{}", err))?;
        // let vec = vec![0];

        self.state_offset = self.size;
        self.state_size = vec.len() as u32;

        self.grow(self.size+vec.len() as u32);

        stable_write(1, &self.state_offset.to_be_bytes());
        stable_write(5, &self.state_size.to_be_bytes());

        stable_write(self.size, &vec);

        // let mut ser = candid::ser::IDLBuilder::new();
        // ser.arg(&t);
        // let vec: Vec<u8> = ser.serialize_to_vec().map_err(|x| String::from("Candid error"))?;

        // self.state_offset = self.size;
        // self.state_size = vec.len() as u32;

        // self.grow(self.size+vec.len() as u32);

        // stable_write(1, &self.state_offset.to_be_bytes());
        // stable_write(5, &self.state_size.to_be_bytes());

        // stable_write(self.size, &vec);

        Ok(())
    }

    pub fn restore_state<T>(&mut self) -> Result<T, String> 
    where
    // T: for<'de> candid::utils::ArgumentDecoder<'de>,
    T: for<'de> serde::Deserialize<'de>,
    {
        let mut u32_buf : [u8;4] = [0,0,0,0]; 
        stable_read(1, &mut u32_buf);
        self.state_offset = u32::from_be_bytes(u32_buf);
        stable_read(5, &mut u32_buf);
        self.state_size = u32::from_be_bytes(u32_buf);

        let mut vec = vec![0; self.state_size as usize];
        stable_read(self.state_offset, &mut vec);

        // let mut de =
        // candid::de::IDLDeserialize::new(&vec).map_err(|e| format!("{:?}", e))?;

        // let res = candid::utils::ArgumentDecoder::decode(&mut de).map_err(|e| format!("{:?}", e))?;
        // let _ = de.done();

        // ic_cdk::storage::stable_restore()
        let data : T = from_slice(&vec).map_err(|err| format!("{}", err))?;
        // Err(String::from("Test"))
        Ok(data)
        // Ok(res)
    }

    // fn read_state_size(&mut self) -> Result<u32, String> {
    //     Ok(0)
    // }
    // fn read_state(&mut self, data: &mut [u8]) -> Result<(), String> {
    //     Ok(())
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_asset() -> Asset {
        Asset {
            name: "example asset".to_string(),
            headers: Vec::default(),
            data: vec![0,1,2,3,4,5,6,7,8,9,10]
        }
    }

    #[test]
    fn store_asset() {
        let mut state = StableStorage::default();
        // let mut state = STORAGE.with(|x| {
        //     x.borrow_mut()
        // });
        let init_result = state.init_storage();
        assert_eq!(init_result, Ok(()));

        let asset = get_asset();

        let result = state.store_asset(&asset);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn read_asset() {
        let mut state = StableStorage::default();
        let init_result = state.init_storage();
        assert_eq!(init_result, Ok(()));

        let asset = get_asset();

        let result = state.store_asset(&asset);
        assert_eq!(result, Ok(()));

        let load_result = state.load_assets();
        assert_eq!(load_result, Ok(()));

        assert_eq!(state.assets.len(), 1);
    }

    #[test]
    fn load_asset() {
        let mut state = StableStorage::default();
        let init_result = state.init_storage();
        assert_eq!(init_result, Ok(()));
        
        let asset = get_asset();

        let result = state.store_asset(&asset);
        assert_eq!(result, Ok(()));

        let load_result = state.get_asset(&asset.name);
        assert_eq!(load_result.is_ok(), true);

        assert_eq!(load_result.unwrap().1.len(), 11);

        // assert_eq!(load_result, Ok(&mut asset.data.data[..]));
    }

    #[test]
    fn reload_get_asset() {
        let mut state = StableStorage::default();
        let init_result = state.init_storage();
        assert_eq!(init_result, Ok(()));
        
        let asset = get_asset();

        let result = state.store_asset(&asset);
        assert_eq!(result, Ok(()));

        let load_result = state.load_assets();
        assert_eq!(load_result, Ok(()));

        let load_result = state.get_asset(&asset.name);
        assert_eq!(load_result.is_ok(), true);

        assert_eq!(load_result.unwrap().1.len(), 11);

        // assert_eq!(load_result, Ok(&mut asset.data.data[..]));
    }
}