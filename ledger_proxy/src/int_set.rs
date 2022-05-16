
use serde::Deserialize;
use candid::CandidType;
use intmap::Keys;
use candid::types::Compound;
// use candid::types::Serializer;
use candid::types::Type;
use serde::de::SeqAccess;
use serde::de::Visitor;

use intmap::IntMap;

use serde::de::Deserializer;
use serde::ser::{Serialize, Serializer, SerializeSeq};

#[derive(Clone, Debug)]
pub struct IntSet(IntMap<()>);

impl IntSet {
    pub fn keys(&self) -> Keys<u64, ()> {
        self.0.keys()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, key: u64) -> Option<&()> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: u64, value: ()) -> bool {
        self.0.insert(key, value)
    }
}

impl Default for IntSet {
    fn default() -> Self {
        Self {
            0: IntMap::new()
        }
    }
}

// impl CandidType for IntSet {
//     fn _ty() -> Type {
//         Type::Vec(Box::new(Type::Nat8))
//     }

//     fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
//     where
//         S: Serializer,
//     {
//         let mut compound = serializer.serialize_vec(self.0.len())?;

//         for e in self.0.keys() {
//             compound.serialize_element(e)?;
//         }

//         Ok(())
//     }
// }

impl Serialize for IntSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.keys() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}


use std::fmt;

struct IntSetDeserializer;

impl<'de> Visitor<'de> for IntSetDeserializer {
    type Value = IntSet;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a vector of u64")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut new_obj = IntSet::default();

        while let Some(key) = seq.next_element()? {
            new_obj.insert(key, ());

            // if let Some(value) = seq.next_element()? {
            // } else {
                // return Err(de::Error::custom(format!(
                    // "Didn't find the right sequence of values in ArrayKeyedMap."
                // )));
            // }
        }

        Ok(new_obj)
    }
}

impl<'de> Deserialize<'de> for IntSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(IntSetDeserializer)
    }
}
