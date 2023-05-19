pub mod events;

use crate::errors::Result;
use std::collections::HashSet;

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Debug)]
pub struct RelayAddress {
    address: String,
}

impl RelayAddress {
    pub fn new(s: String) -> Result<RelayAddress> {
        if s.is_empty() {
            return Err("empty token".into());
        }

        if !(s.starts_with("wss://") || s.starts_with("ws://")) {
            return Err(format!("invalid relay address: '{s}'").into());
        }

        Ok(RelayAddress { address: s })
    }
}

impl AsRef<str> for RelayAddress {
    fn as_ref(&self) -> &str {
        &self.address
    }
}

#[derive(Clone)]
pub struct Locale {
    locale: String,
}

impl Locale {
    pub fn new(s: String) -> Result<Locale> {
        if s.is_empty() {
            return Err("empty token".into());
        }

        Ok(Locale { locale: s })
    }
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        &self.locale
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PubKey {
    key: nostr::key::XOnlyPublicKey,
}

impl PubKey {
    pub fn new(key: nostr::key::XOnlyPublicKey) -> Self {
        Self { key }
    }

    pub fn new_from_hex(hex: &str) -> Result<Self> {
        let v = hex::decode(hex)?;
        let key = nostr::key::XOnlyPublicKey::from_slice(&v)?;
        Ok(Self { key })
    }

    pub fn hex(&self) -> String {
        format!("{:x}", self.key)
    }
}

pub struct Registration {
    pub_key: PubKey,
    apns_token: APNSToken,
    relays: Vec<RelayAddress>,
    locale: Locale,
}

impl Registration {
    pub fn new(
        pub_key: PubKey,
        apns_token: APNSToken,
        relays: Vec<RelayAddress>,
        locale: Locale,
    ) -> Result<Registration> {
        if relays.is_empty() {
            return Err("empty relays".into());
        }

        let mut relays_set: HashSet<&RelayAddress> = HashSet::new();
        for relay_address in &relays {
            if !relays_set.insert(relay_address) {
                return Err("duplicate relay".into());
            }
        }

        Ok(Registration {
            pub_key,
            apns_token,
            relays,
            locale,
        })
    }

    pub fn pub_key(&self) -> PubKey {
        self.pub_key.clone()
    }

    pub fn apns_token(&self) -> APNSToken {
        self.apns_token.clone()
    }

    pub fn locale(&self) -> Locale {
        self.locale.clone()
    }

    pub fn relays(&self) -> Vec<RelayAddress> {
        self.relays.clone()
    }
}

#[derive(Clone)]
pub struct APNSToken {
    token: String,
}

impl APNSToken {
    pub fn new(s: String) -> Result<APNSToken> {
        if s.is_empty() {
            return Err("empty token".into());
        }

        Ok(APNSToken { token: s })
    }
}

impl AsRef<str> for APNSToken {
    fn as_ref(&self) -> &str {
        &self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod pub_key_tests {
        use super::*;
        use crate::fixtures;

        #[test]
        fn hex() -> Result<()> {
            let hex = "78c5260d54a9c09065c698950c0bb97b9b03d3c103aa0d5507c902f70e7088af";
            let key = PubKey::new_from_hex(hex)?;
            assert_eq!(key.hex(), hex);
            Ok(())
        }

        #[test]
        fn new_from_hex_loads_pub_key() -> Result<()> {
            let pub_key = fixtures::some_pub_key();
            let unmarshaled_pub_key = PubKey::new_from_hex(pub_key.hex().as_ref())?;
            assert_eq!(pub_key, unmarshaled_pub_key);

            Ok(())
        }
    }

    #[cfg(test)]
    mod registration_tests {
        use super::*;
        use crate::fixtures;

        #[test]
        fn relay_addresses_can_not_be_duplicate() -> Result<()> {
            let pub_key = fixtures::some_pub_key();
            let apns_token = fixtures::some_apns_token();
            let locale = fixtures::some_locale();
            let relay_address_1 = fixtures::some_relay_address();
            let relay_address_2 = fixtures::some_relay_address();

            match Registration::new(
                pub_key.clone(),
                apns_token.clone(),
                vec![relay_address_1.clone(), relay_address_2.clone()],
                locale.clone(),
            ) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }

            match Registration::new(
                pub_key.clone(),
                apns_token.clone(),
                vec![relay_address_1.clone(), relay_address_1.clone()],
                locale.clone(),
            ) {
                Ok(_) => return Err("expected an error".into()),
                Err(err) => {
                    assert_eq!(err.to_string(), "duplicate relay")
                }
            }

            Ok(())
        }
    }

    #[test]
    fn it_works() {
        match APNSToken::new(String::from("")) {
            Ok(_) => panic!("constructor should have returned an error"),
            Err(error) => assert_eq!(error.to_string(), String::from("empty token")),
        }
    }
}
