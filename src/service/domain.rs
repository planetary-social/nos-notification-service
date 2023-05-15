pub mod events;

use std::error::Error;

pub struct RelayAddress {
    address: String,
}

impl RelayAddress {
    pub fn new(s: String) -> Result<RelayAddress, Box<dyn Error>> {
        if s.is_empty() {
            return Result::Err(Into::into("empty token"));
        }

        return Ok(RelayAddress { address: s });
    }
}

pub struct Locale {
    locale: String,
}

impl Locale {
    pub fn new(s: String) -> Result<Locale, Box<dyn Error>> {
        if s.is_empty() {
            return Result::Err(Into::into("empty token"));
        }

        return Ok(Locale { locale: s });
    }
}

pub struct Registration {
    // todo add key
    apns_token: APNSToken,
    relays: Vec<RelayAddress>,
    locale: Locale,
}

impl Registration {
    pub fn new(
        apns_token: APNSToken,
        relays: Vec<RelayAddress>,
        locale: Locale,
    ) -> Result<Registration, Box<dyn Error>> {
        if relays.is_empty() {
            return Result::Err(Into::into("empty relays"));
        }

        return Ok(Registration {
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
    pub fn new(s: String) -> Result<APNSToken, String> {
        if s.is_empty() {
            return Result::Err(String::from("empty token"));
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
        match APNSToken::new(String::from("")) {
            Ok(_) => panic!("constructor should have returned an error"),
            Err(error) => assert_eq!(error, String::from("empty token")),
        }
    }
}
