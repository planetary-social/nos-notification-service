pub mod events;

use crate::errors::Result;

pub struct RelayAddress {
    address: String,
}

impl RelayAddress {
    pub fn new(s: String) -> Result<RelayAddress> {
        if s.is_empty() {
            return Result::Err(Into::into("empty token"));
        }

        if !(s.starts_with("wss://") || s.starts_with("ws://")) {
            return Result::Err(Into::into(format!("invalid relay address: '{}'", s)));
        }

        return Ok(RelayAddress { address: s });
    }
}

pub struct Locale {
    locale: String,
}

impl Locale {
    pub fn new(s: String) -> Result<Locale> {
        if s.is_empty() {
            return Result::Err(Into::into("empty token"));
        }

        return Ok(Locale { locale: s });
    }
}

pub struct PubKey {
    key: nostr::key::XOnlyPublicKey,
}

impl PubKey {
    pub fn new(key: nostr::key::XOnlyPublicKey) -> Result<Self> {
        return Ok(Self { key });
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
            return Result::Err(Into::into("empty relays"));
        }

        return Ok(Registration {
            pub_key,
            apns_token,
            relays,
            locale,
        });
    }

    pub fn apns_token(&self) -> APNSToken {
        return self.apns_token.clone();
    }
}

#[derive(Clone)]
pub struct APNSToken {
    token: String,
}

impl APNSToken {
    pub fn new(s: String) -> Result<APNSToken> {
        if s.is_empty() {
            return Err(String::from("empty token"))?;
        }

        return Ok(APNSToken { token: s });
    }
}

impl Into<String> for APNSToken {
    fn into(self) -> String {
        return self.token;
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
