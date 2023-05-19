use crate::service::domain;

pub fn some_relay_address() -> domain::RelayAddress {
    return domain::RelayAddress::new(String::from("wss://some-relay-address")).unwrap();
}
