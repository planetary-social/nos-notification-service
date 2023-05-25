pub mod events;

use crate::errors::Result;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct PubKey {
    key: nostr::key::XOnlyPublicKey,
}

impl PubKey {
    pub fn new(key: nostr::key::XOnlyPublicKey) -> Self {
        Self { key }
    }

    pub fn hex(&self) -> String {
        format!("{:x?}", self.key)
    }
}

pub struct Registration {
    pub_key: PubKey,
    apns_token: APNSToken,
    relays: Vec<RelayAddress>, // todo add a type, remove dupes, prevent empty relays?
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

    #[test]
    fn it_works() {
        //match APNSToken::new(String::from("")) {
        //    Ok(_) => panic!("constructor should have returned an error"),
        //    Err(error) => assert_eq!(error, String::from("empty token")),
        //    }
    }
}
