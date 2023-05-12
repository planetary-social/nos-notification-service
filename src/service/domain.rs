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

        return Result::Err(Into::into("not implemented"));
    }
}

pub struct Locale {}

impl Locale {
    pub fn new(s: String) -> Result<Locale, Box<dyn Error>> {
        if s.is_empty() {
            return Result::Err(Into::into("empty token"));
        }

        return Result::Err(Into::into("not implemented"));
    }
}

pub struct Registration {
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
}

pub struct APNSToken {
    s: String,
}

impl APNSToken {
    pub fn new(s: String) -> Result<APNSToken, String> {
        if s.is_empty() {
            return Result::Err(String::from("empty token"));
        }

        return Result::Err(String::from("not implemented"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let tk = Token::new(String::from(""));
        match tk {
            Ok(_) => panic!("constructor should have returned an error"),
            Err(error) => assert_eq!(error, String::from("empty token")),
        }
    }
}
