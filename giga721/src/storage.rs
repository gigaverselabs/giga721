use common::HeaderField;
use ic_cdk::api::stable::StableMemoryError;
use std::collections::HashMap;
use std::rc::Rc;

use ic_cdk::export::candid::CandidType;
use ic_cdk::print;

use serde_cbor::{from_slice, to_vec};

use serde::{Deserialize, Serialize};

use std::cell::RefCell;

use common::rc_bytes::RcBytes;
use std::io;

#[cfg(test)]
use crate::testing::{stable_grow, stable_read, stable_size, stable_write, trap};

#[cfg(not(test))]
use ic_cdk::api::stable::{stable_grow, stable_read, stable_size, stable_write};

#[cfg(not(test))]
use ic_cdk::api::trap;

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct Asset {
    pub name: String,
    pub content_type: String,
    pub data: RcBytes,
    // pub headers: Vec<HeaderField>,
    // pub data: Vec<u8>
}

#[derive(Serialize, Deserialize, Default)]
pub struct StableStorage {
    pub assets: HashMap<String, (u32, u32, Vec<HeaderField>)>, //Stores offset and size of asset

    /// Stores size of all assets along with data pages
    pub size: u32,

    /// Keeps information about offset for state
    pub state_offset: u32,
    /// Stored state size in bytes
    pub state_size: u32,
}

thread_local! {
    pub static STORAGE: Rc<RefCell<StableStorage>> = Rc::new(RefCell::new(StableStorage::default()));
}

impl StableStorage {
    pub fn get() -> Rc<RefCell<StableStorage>> {
        STORAGE.with(|x| x.clone())
    }

    /// Initialize stable storage data structure, use with caution, this will wipe all data in st able storage!
    pub fn init_storage(&mut self) -> Result<(), ()> {
        match self.grow(1) {
            Err(_) => Err(()),
            Ok(_) => {
                stable_write(0, &[0;4]); //Number of assets
                stable_write(4, &[0;4]); //State offset
                stable_write(8, &[0;4]); //State size
                self.size = 12; //3*4

                Ok(())
            }
        } //initialize stable storage if necessary
    }

    /// Ensures that there is enough space in stable memory
    fn grow(&mut self, size: u32) -> Result<(), StableMemoryError> {
        if self.size + size > stable_size() << 16 {
            stable_grow((size >> 16) + 1)?;
        }

        Ok(())
    }

    fn stable_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), String> {
        self.grow(offset+(buf.len() as u32)).map_err(|_| String::from("Stable memory error"))?;
        stable_write(offset, buf);
        Ok(())
    }

    fn stable_read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), String> {
        let size = (stable_size() as usize) << 16;
        
        if size < (offset as usize) + buf.len() { 
            return Err(String::from("Trying to read from outside of stable memory")) 
        }
    
        stable_read(offset, buf);

        Ok(())
    }

    #[inline(always)]
    fn write_u32(&mut self, offset: u32, data: u32) -> Result<u32, String> {
        self.stable_write(offset, &data.to_be_bytes())?;
        Ok(4)
    }

    #[inline(always)]
    fn read_u32(&mut self, offset: u32, data: &mut u32) -> Result<u32, String> {
        let mut u32_buf: [u8; 4] = [0, 0, 0, 0];
        self.stable_read(offset, &mut u32_buf)?;
        *data = u32::from_be_bytes(u32_buf);

        Ok(4)
    }

    fn write_str(&mut self, offset: u32, data: &str) -> Result<u32, String> {
        let size = self.write_u32(offset, data.len() as u32)?;

        let bytes = data.as_bytes();

        self.stable_write(offset+size, &bytes)?;
        
        Ok(size+(bytes.len() as u32))
    }

    fn read_str(&mut self, offset: u32, data: &mut String) -> Result<u32, String> {
        let mut str_size : u32 = 0;
        let size = self.read_u32(offset, &mut str_size)?;

        let mut str_vec = vec![0; str_size as usize];
        self.stable_read(offset+size, &mut str_vec)?;

        *data = String::from_utf8(str_vec).map_err(|_| String::from("Error on utf8 conversion of asset name"))?;

        Ok(size+str_size)
    }

    fn write_bytes(&mut self, offset: u32, data: &[u8]) -> Result<u32, String> {
        let size = self.write_u32(offset, data.len() as u32)?;
        self.stable_write(offset+size, data)?;

        Ok(size+(data.len() as u32))
    }

    /// Stores asset in stable memory, returns err if storage is not initialized
    /// data structure: name_length, name_utf8, header_length, headers_cbor, data_length, data_array
    pub fn store_asset(&mut self, asset: &Asset) -> Result<(), String> {
        //Prepare headers
        let mut headers: Vec<(String, String)> = Vec::default();
        headers.push((String::from("Content-Type"), asset.content_type.clone()));

        //Serialize headers to vec<u8>
        let vec = to_vec(&headers).map_err(|err| format!("{}", err))?;
        let offset = self.size;

        //Write asset name
        self.size += self.write_str(self.size, &asset.name)?;

        //Write headers data
        self.size += self.write_bytes(self.size, &vec)?;

        //Write data
        self.size += self.write_bytes(self.size, &asset.data)?;

        //Todo: requires fixing
        self.assets.insert(
            asset.name.clone(),
            (offset, asset.data.len() as u32, headers.clone())
        );

        //Update number of stored assets
        self.write_u32(0, self.assets.len() as u32)?;
        //Update size of data
        self.write_u32(4, offset)?;

        return Ok(());
    }

    /// Reads asset data from stable memory
    pub fn get_asset(&mut self, name: &str) -> Result<(Vec<HeaderField>, RcBytes), String> {
        let (offset, size, headers) = self
            .assets
            .get(name)
            .ok_or_else(|| format!("Asset not found {}", name))?;

        let mut buf = vec![0; *size as usize];

        stable_read(*offset, &mut buf);

        let bytes = RcBytes::from(buf);
        Ok((headers.clone(), bytes))
    }

    /// Load assets information from the stable storage, it does not load all asset data in to cache, only names and headers
    pub fn load_assets(&mut self) -> Result<(), String> {
        if stable_size() == 0 {
            return Err(String::from("No data in stable storage"));
        }

        //Clean AssetStorage
        self.assets.clear();

        //Load number of items to process, u8
        let mut items: u32 = 0;
        self.read_u32(0, &mut items)?;

        //Skip state info
        let mut offset = 12;

        // print(format!("Items in stable storage: {}", items));

        for _i in 0..items {

            // print(format!("Loading item: {}", _i));
            
            //Read asset name
            let mut name = String::default();
            offset += self.read_str(offset, &mut name)?;

            // print(format!("Name: {}", name));
            

            //Read headers of assets
            let mut headers_size: u32 = 0;
            offset += self.read_u32(offset, &mut headers_size)?;

            // print(format!("Headers size: {}", headers_size));


            let mut headers_vec = vec![0; headers_size as usize];
            stable_read(offset, &mut headers_vec);
            offset += headers_size;

            let headers: Vec<HeaderField> =
                from_slice(&headers_vec).map_err(|_| String::from("Could not parse data"))?;

            //Read data length
            // stable_read(offset, &mut u32_buf);
            // offset += 4;
            // let data_size = u32::from_be_bytes(u32_buf);
            let mut data_size: u32 = 0;
            offset += self.read_u32(offset, &mut data_size)?;

            // print(format!("Data size: {}", data_size));


            self.assets
                .insert(name.clone(), (offset, data_size, headers));

            //Skip loading data, move to next item
            offset += data_size;

            // //test load data
            // let mut data = vec![0;data_size as usize];
            // stable_read(offset, &mut data);
        }

        self.size = offset;

        return Ok(());
    }

    /// Serializes objec with serde_cbor and saves it to stable storage
    pub fn store_state<T>(&mut self, t: T) -> Result<(), String>
    where
        T: serde::Serialize,
    {
        let vec = to_vec(&t).map_err(|err| format!("{}", err))?;

        self.state_offset = self.size;
        self.state_size = vec.len() as u32;

        self.grow(self.size + vec.len() as u32)
            .map_err(|_| String::from("Could not grow stable storage!"))?;

        stable_write(4, &self.state_offset.to_be_bytes());
        stable_write(8, &self.state_size.to_be_bytes());

        stable_write(self.size, &vec);

        Ok(())
    }

    /// Reads object from stable storage
    pub fn restore_state<T>(&mut self) -> Result<T, String>
    where
        // T: for<'de> candid::utils::ArgumentDecoder<'de>,
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut offset: u32 = 0;
        let mut size: u32 = 0;
        self.read_u32(4, &mut offset)?;
        self.read_u32(8, &mut size)?;

        self.state_offset = offset;
        self.state_size = size;

        // print(format!("offset: {} , size: {}",self.state_offset, self.state_size));

        let mut vec = vec![0; self.state_size as usize];
        stable_read(self.state_offset, &mut vec);

        let data: T = from_slice(&vec).map_err(|err| format!("Err while parsing: {}", err))?;
        Ok(data)
    }
}

/// A reader to the stable memory. Keeps an offset and reads off stable memory
/// consecutively.
pub struct StableReader {
    /// The offset of the next write.
    pub offset: usize,
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
}

impl io::Read for StableReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unexpected error."))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::get_asset;

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
    fn load_assets_test() {
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
    fn get_asset_test() {
        let mut state = StableStorage::default();
        let init_result = state.init_storage();
        assert_eq!(init_result, Ok(()));

        let asset = get_asset();
        state.store_asset(&asset).unwrap();

        let asset_result = state.get_asset(&asset.name);

        assert!(asset_result.is_ok());
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

        assert_eq!(load_result.unwrap().1.len(), asset.data.len());

        // assert_eq!(load_result, Ok(&mut asset.data.data[..]));
    }

    #[test]
    fn reload_get_asset() {
        let mut state = StableStorage::default();
        let init_result = state.init_storage();
        assert_eq!(init_result, Ok(()));
        let asset = get_asset();

        //Add asset to stable storage
        let result = state.store_asset(&asset);
        assert_eq!(result, Ok(()));

        //Loads assets from stable storage
        let load_result = state.load_assets();
        assert_eq!(load_result, Ok(()));

        //Get assetd data
        let load_result = state.get_asset(&asset.name);
        assert_eq!(load_result.is_ok(), true);

        assert_eq!(load_result.unwrap().1.len(), asset.data.len());

        // assert_eq!(load_result, Ok(&mut asset.data.data[..]));
    }
}
