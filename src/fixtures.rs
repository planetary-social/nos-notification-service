use crate::service::domain;
use rand::{distributions::Alphanumeric, Rng};

pub fn some_relay_address() -> domain::RelayAddress {
    return domain::RelayAddress::new(String::from(format!(
        "wss://some-relay-address-{}",
        random_string()
    )))
    .unwrap();
}

pub fn some_pub_key() -> domain::PubKey {
    let (_sk, pk) = nostr::secp256k1::generate_keypair(&mut rand::rngs::OsRng {});
    return domain::PubKey::new(nostr::key::XOnlyPublicKey::from(pk));
}

pub fn some_apns_token() -> domain::APNSToken {
    return domain::APNSToken::new(String::from("apns_token")).unwrap();
}

pub fn some_locale() -> domain::Locale {
    return domain::Locale::new(String::from("some locale")).unwrap();
}

fn random_string() -> String {
    return rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
}
