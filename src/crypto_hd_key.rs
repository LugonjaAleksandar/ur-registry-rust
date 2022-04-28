use std::collections::BTreeMap;
use hex::FromHex;
use serde_cbor::{from_slice, to_vec, Value};
use serde_cbor::value::from_value;
use crate::crypto_coin_info::CryptoCoinInfo;
use crate::crypto_key_path::CryptoKeyPath;
use crate::registry_types::{CRYPTO_HDKEY, RegistryType};
use crate::traits::{RegistryItem, To, From};
use crate::types::{Bytes, Fingerprint};

const IS_MASTER: i128 = 1;
const IS_PRIVATE: i128 = 2;
const KEY_DATA: i128 = 3;
const CHAIN_CODE: i128 = 4;
const USE_INFO: i128 = 5;
const ORIGIN: i128 = 6;
const CHILDREN: i128 = 7;
const PARENT_FINGERPRINT: i128 = 8;
const NAME: i128 = 9;
const NOTE: i128 = 10;

#[derive(Clone, Debug, Default)]
pub struct CryptoHDKey {
    is_master: Option<bool>,
    is_private_key: Option<bool>,
    key: Option<Bytes>,
    chain_code: Option<Bytes>,
    use_info: Option<CryptoCoinInfo>,
    origin: Option<CryptoKeyPath>,
    children: Option<CryptoKeyPath>,
    parent_fingerprint: Option<Fingerprint>,
    name: Option<String>,
    note: Option<String>,
}

impl CryptoHDKey {
    pub fn new_master_key(key: Bytes, chain_code: Bytes) -> CryptoHDKey {
        CryptoHDKey { is_master: Some(true), key: Some(key), chain_code: Some(chain_code), ..Default::default() }
    }

    pub fn new_extended_key(
        is_private_key: Option<bool>,
        key: Bytes,
        chain_code: Option<Bytes>,
        use_info: Option<CryptoCoinInfo>,
        origin: Option<CryptoKeyPath>,
        children: Option<CryptoKeyPath>,
        parent_fingerprint: Option<Fingerprint>,
        name: Option<String>,
        note: Option<String>,
    ) -> CryptoHDKey {
        CryptoHDKey {
            is_master: Some(false),
            is_private_key,
            key: Some(key),
            chain_code,
            use_info,
            origin,
            children,
            parent_fingerprint,
            name,
            note,
        }
    }

    pub fn is_master(&self) -> bool {
        self.is_master.clone().unwrap_or(false)
    }
    pub fn is_private_key(&self) -> bool {
        self.is_private_key.clone().unwrap_or(false)
    }
    pub fn get_key(&self) -> Option<Vec<u8>> {
        self.key.clone()
    }
    pub fn get_chain_code(&self) -> Option<Vec<u8>> {
        self.chain_code.clone()
    }
    pub fn get_use_info(&self) -> Option<CryptoCoinInfo> {
        self.use_info.clone()
    }
    pub fn get_origin(&self) -> Option<CryptoKeyPath> {
        self.origin.clone()
    }
    pub fn get_children(&self) -> Option<CryptoKeyPath> {
        self.children.clone()
    }
    pub fn get_parent_fingerprint(&self) -> Option<Fingerprint> {
        self.parent_fingerprint.clone()
    }
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
    pub fn get_note(&self) -> Option<String> {
        self.note.clone()
    }

    pub fn get_bip32_key(&self) -> String {
        let mut version: Bytes = vec![0; 4];
        let mut depth: u8 = 0;
        let mut index: u32 = 0;
        let mut parent_fingerprint: Fingerprint = self.parent_fingerprint.unwrap_or([0, 0, 0, 0]);
        let mut chain_code = self.get_chain_code().unwrap_or(vec![0; 32]);
        let mut key = self.get_key().unwrap_or(vec![0; 32]);
        if self.is_master() {
            version = vec![0x04, 0x88, 0xAD, 0xE4];
            depth = 0;
            index = 0;
        } else {
            match self.get_origin() {
                Some(x) => {
                    depth = x.get_components().len() as u8;
                    index = x.get_components().last().unwrap().get_canonical_index().unwrap_or(0);
                }
                None => {},
            };
            version = match self.is_private_key() {
                true => vec![0x04, 0x88, 0xAD, 0xE4],
                false => vec![0x04, 0x88, 0xB2, 0x1E],
            }
        }
        let mut output = vec![];
        output.append(version.as_mut()); // 4
        output.append(depth.to_be_bytes().to_vec().as_mut()); // 1
        output.append(parent_fingerprint.to_vec().as_mut()); // 4
        output.append(index.to_be_bytes().to_vec().as_mut()); // 4
        output.append(chain_code.as_mut()); //32
        output.append(key.as_mut()); //33
        bs58::encode(output).with_check().into_string()
    }
}

impl RegistryItem for CryptoHDKey {
    fn get_registry_type() -> RegistryType<'static> {
        CRYPTO_HDKEY
    }
}

impl To for CryptoHDKey {
    fn to_cbor(&self) -> Value {
        let mut map: BTreeMap<Value, Value> = BTreeMap::new();
        if self.is_master() {
            map.insert(Value::Integer(IS_MASTER), Value::Bool(self.is_master()));
            map.insert(Value::Integer(KEY_DATA), Value::Bytes(self.key.clone().unwrap()));
            map.insert(Value::Integer(CHAIN_CODE), Value::Bytes(self.chain_code.clone().unwrap()));
        } else {
            match self.is_private_key {
                Some(x) => { map.insert(Value::Integer(IS_PRIVATE), Value::Bool(x)); }
                None => {}
            }
            map.insert(Value::Integer(KEY_DATA), Value::Bytes(self.key.clone().unwrap()));
            match &self.chain_code {
                Some(x) => { map.insert(Value::Integer(CHAIN_CODE), Value::Bytes(x.clone())); }
                None => {}
            }
            match &self.use_info {
                Some(x) => {
                    map.insert(
                        Value::Integer(USE_INFO),
                        Value::Tag(
                            CryptoCoinInfo::get_registry_type().get_tag() as u64,
                            Box::new(x.to_cbor()),
                        ),
                    );
                }
                None => {}
            }
            match &self.origin {
                Some(x) => {
                    map.insert(
                        Value::Integer(ORIGIN),
                        Value::Tag(
                            CryptoKeyPath::get_registry_type().get_tag() as u64,
                            Box::new(x.to_cbor()),
                        ),
                    );
                }
                None => {}
            }
            match &self.children {
                Some(x) => {
                    map.insert(
                        Value::Integer(ORIGIN),
                        Value::Tag(
                            CryptoKeyPath::get_registry_type().get_tag() as u64,
                            Box::new(x.to_cbor()),
                        ),
                    );
                }
                None => {}
            }
            match self.parent_fingerprint {
                Some(x) => {
                    map.insert(Value::Integer(PARENT_FINGERPRINT), Value::Integer(u32::from_be_bytes(x) as i128));
                }
                None => {}
            }
            match &self.name {
                Some(x) => {
                    map.insert(Value::Integer(NAME), Value::Text(x.clone()));
                }
                None => {}
            }
            match &self.note {
                Some(x) => {
                    map.insert(Value::Integer(NOTE), Value::Text(x.clone()));
                }
                None => {}
            }
        }
        Value::Map(map)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let value = self.to_cbor();
        to_vec(&value).unwrap()
    }
}

impl From<CryptoHDKey> for CryptoHDKey {
    fn from_cbor(cbor: Value) -> Result<CryptoHDKey, String> {
        let map: BTreeMap<Value, Value> = match from_value(cbor) {
            Ok(x) => x,
            Err(e) => return Err(e.to_string())
        };
        let is_master = match map.get(&Value::Integer(IS_MASTER)) {
            Some(x) => {
                match from_value::<bool>(x.clone()) {
                    Ok(x) => Some(x),
                    Err(e) => return Err(e.to_string()),
                }
            }
            None => None,
        };
        match is_master {
            Some(true) => {
                let key = match map.get(&Value::Integer(KEY_DATA)) {
                    Some(Value::Bytes(x)) => Some(x.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.key_data".to_string()),
                    None => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]key data is required for crypto-hdkey when it is a master key".to_string()),
                };
                let chain_code = match map.get(&Value::Integer(CHAIN_CODE)) {
                    Some(Value::Bytes(x)) => Some(x.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.chain_code".to_string()),
                    None => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]chain code is required for crypto-hdkey when it is a master key".to_string())
                };
                Ok(CryptoHDKey { is_master: Some(true), key, chain_code, ..Default::default() })
            }
            _is_master => {
                let is_private_key = match map.get(&Value::Integer(IS_PRIVATE)) {
                    Some(Value::Bool(x)) => Some(x.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.is_private_key".to_string()),
                    None => None,
                };
                let key = match map.get(&Value::Integer(KEY_DATA)) {
                    Some(Value::Bytes(x)) => Some(x.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.key_data".to_string()),
                    None => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]key data is required for crypto-hdkey when it is a hd key".to_string()),
                };
                let chain_code = match map.get(&Value::Integer(CHAIN_CODE)) {
                    Some(Value::Bytes(x)) => Some(x.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.chain_code".to_string()),
                    None => None,
                };
                let use_info = match map.get(&Value::Integer(USE_INFO)) {
                    Some(Value::Tag(x, value)) => {
                        if x.clone() != CryptoCoinInfo::get_registry_type().get_tag() as u64 {
                            return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.use_info".to_string());
                        }
                        match CryptoCoinInfo::from_cbor(*value.clone()) {
                            Ok(value) => Some(value),
                            Err(e) => return Err(e),
                        }
                    }
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.use_info".to_string()),
                    None => None
                };
                let origin = match map.get(&Value::Integer(ORIGIN)) {
                    Some(Value::Tag(tag, value)) => {
                        if tag.clone() != CryptoKeyPath::get_registry_type().get_tag() as u64 {
                            return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.origin".to_string());
                        }
                        match CryptoKeyPath::from_cbor(*value.clone()) {
                            Ok(value) => Some(value),
                            Err(e) => return Err(e),
                        }
                    }
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.origin".to_string()),
                    None => None,
                };
                let children = match map.get(&Value::Integer(CHILDREN)) {
                    Some(Value::Tag(tag, value)) => {
                        if tag.clone() != CryptoKeyPath::get_registry_type().get_tag() as u64 {
                            return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.children".to_string());
                        }
                        match CryptoKeyPath::from_cbor(*value.clone()) {
                            Ok(value) => Some(value),
                            Err(e) => return Err(e),
                        }
                    }
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.children".to_string()),
                    None => None,
                };
                let parent_fingerprint = match map.get(&Value::Integer(PARENT_FINGERPRINT)) {
                    Some(Value::Integer(x)) => Some(u32::to_be_bytes(x.clone() as u32)),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.parent_fingerprint".to_string()),
                    None => None,
                };
                let name = match map.get(&Value::Integer(NAME)) {
                    Some(Value::Text(name)) => Some(name.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.name".to_string()),
                    None => None,
                };
                let note = match map.get(&Value::Integer(NOTE)) {
                    Some(Value::Text(note)) => Some(note.clone()),
                    Some(_) => return Err("[ur-registry-rust][crypto-hdkey][from_cbor]received unexpected value when parsing data to crypto-hdkey.note".to_string()),
                    None => None,
                };
                Ok(CryptoHDKey { is_master: _is_master, is_private_key, key, chain_code, use_info, origin, children, name, note, parent_fingerprint, ..Default::default() })
            }
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<CryptoHDKey, String> {
        let value: Value = match from_slice(bytes.as_slice()) {
            Ok(x) => x,
            Err(e) => return Err(e.to_string()),
        };
        CryptoHDKey::from_cbor(value)
    }
}

#[cfg(test)]
mod tests {
    use hex;
    use hex::{FromHex};
    use crate::crypto_coin_info::{CoinType, CryptoCoinInfo, Network};
    use crate::crypto_hd_key::CryptoHDKey;
    use crate::crypto_key_path::{CryptoKeyPath, PathComponent};
    use crate::traits::{From, To};

    #[test]
    fn test_encode() {
        let master_key = CryptoHDKey::new_master_key(
            Vec::from_hex("00e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35").unwrap(),
            Vec::from_hex("873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508").unwrap(),
        );
        assert_eq!(
            "A301F503582100E8F32E723DECF4051AEFAC8E2C93C9C5B214313817CDB01A1494B917C8436B35045820873DFF81C02F525623FD1FE5167EAC3A55A049DE3D314BB42EE227FFED37D508",
            hex::encode(master_key.to_bytes()).to_uppercase()
        );

        let hd_key = CryptoHDKey::new_extended_key(
            None,
            Vec::from_hex("026fe2355745bb2db3630bbc80ef5d58951c963c841f54170ba6e5c12be7fc12a6").unwrap(),
            Some(Vec::from_hex("ced155c72456255881793514edc5bd9447e7f74abb88c6d6b6480fd016ee8c85").unwrap()),
            Some(CryptoCoinInfo::new(None, Some(Network::TestNet))),
            Some(CryptoKeyPath::new(
                vec![
                    PathComponent::new(Some(44), true).unwrap(),
                    PathComponent::new(Some(1), true).unwrap(),
                    PathComponent::new(Some(1), true).unwrap(),
                    PathComponent::new(Some(0), false).unwrap(),
                    PathComponent::new(Some(1), false).unwrap(),
                ],
                None,
                None,
            )),
            None,
            Some([0xe9, 0x18, 0x1c, 0xf3]),
            None,
            None,
        );

        assert_eq!(
            "A5035821026FE2355745BB2DB3630BBC80EF5D58951C963C841F54170BA6E5C12BE7FC12A6045820CED155C72456255881793514EDC5BD9447E7F74ABB88C6D6B6480FD016EE8C8505D90131A1020106D90130A1018A182CF501F501F500F401F4081AE9181CF3",
            hex::encode(hd_key.to_bytes()).to_uppercase()
        )
    }

    #[test]
    fn test_decode() {
        let master_key = CryptoHDKey::from_bytes(Vec::from_hex("A301F503582100E8F32E723DECF4051AEFAC8E2C93C9C5B214313817CDB01A1494B917C8436B35045820873DFF81C02F525623FD1FE5167EAC3A55A049DE3D314BB42EE227FFED37D508").unwrap()).unwrap();
        assert_eq!("00e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35", hex::encode(master_key.key.unwrap()));
        assert_eq!("873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508", hex::encode(master_key.chain_code.unwrap()));

        let hd_key = CryptoHDKey::from_bytes(Vec::from_hex("A5035821026FE2355745BB2DB3630BBC80EF5D58951C963C841F54170BA6E5C12BE7FC12A6045820CED155C72456255881793514EDC5BD9447E7F74ABB88C6D6B6480FD016EE8C8505D90131A1020106D90130A1018A182CF501F501F500F401F4081AE9181CF3").unwrap()).unwrap();
        assert_eq!("026fe2355745bb2db3630bbc80ef5d58951c963c841f54170ba6e5c12be7fc12a6", hex::encode(hd_key.key.clone().unwrap()));
        assert_eq!("ced155c72456255881793514edc5bd9447e7f74abb88c6d6b6480fd016ee8c85", hex::encode(hd_key.chain_code.clone().unwrap()));
        assert_eq!(false, hd_key.is_master());
        assert_eq!(false, hd_key.is_private_key());
        assert_eq!(CoinType::Bitcoin, hd_key.get_use_info().unwrap().get_coin_type());
        assert_eq!(Network::TestNet, hd_key.get_use_info().unwrap().get_network());
        assert_eq!("44'/1'/1'/0/1", hd_key.get_origin().unwrap().get_path().unwrap());
        assert_eq!([0xe9, 0x18, 0x1c, 0xf3], hd_key.get_parent_fingerprint().unwrap());
        assert_eq!("xpub6H8Qkexp9BdSgEwPAnhiEjp7NMXVEZWoAFWwon5mSwbuPZMfSUTpPwAP1Q2q2kYMRgRQ8udBpEj89wburY1vW7AWDuYpByteGogpB6pPprX", hd_key.get_bip32_key());
    }
}