use crate::errors::Result;
use crate::service::app::common;
//use std::sync::mpsc::{channel, Sender, Receiver};
use crate::service::domain;
use std::collections::HashMap;
use std::thread;

pub struct Downloader<T> {
    transaction_provider: T,
    relay_downloaders: HashMap<domain::RelayAddress, RelayDownloader>,
}

impl<T> Downloader<T> {
    pub fn new(transaction_provider: T) -> Self {
        Self {
            transaction_provider,
            relay_downloaders: HashMap::new(),
        }
    }
}

impl<'a, T> Downloader<T>
where
    T: common::TransactionProvider + Clone + Sync + Send + 'a,
{
    pub fn run(&mut self, scope: &'a thread::Scope<'a, '_>) -> Result<()> {
        let transaction = self.transaction_provider.start_transaction()?;
        let adapters = transaction.adapters();
        let registrations = adapters.registrations.borrow();

        for relay in registrations.get_relays()? {
            let v = RelayDownloader::new(scope, relay.clone(), self.transaction_provider.clone());
            self.relay_downloaders.insert(relay.clone(), v);
        }

        // todo update the list in a loop

        Err("not implemented".into())
    }
}

pub struct RelayDownloader {}

impl RelayDownloader {
    fn new<'a, 'scope, T>(
        scope: &'scope thread::Scope<'scope, '_>,
        relay: domain::RelayAddress,
        transaction_provider: T,
    ) -> RelayDownloader
    where
        T: common::TransactionProvider + Clone + Sync + Send + 'scope,
        'scope: 'a,
    {
        let v = Self {
            //transaction_provider.clone(),
            //relay.clone(),
        };

        //let relay = relay.clone();
        //let transaction_provider = transaction_provider.clone();
        scope.spawn(|| {
            RelayDownloaderRunner::new(relay, transaction_provider).run();
        });

        v
    }
}

pub struct RelayDownloaderRunner<T> {
    transaction_provider: T,
    relay: domain::RelayAddress,
}

impl<T> RelayDownloaderRunner<T>
where
    T: common::TransactionProvider,
{
    fn new(relay: domain::RelayAddress, transaction_provider: T) -> RelayDownloaderRunner<T> {
        Self {
            transaction_provider,
            relay,
        }
    }

    fn run(&self) {
        loop {
            match self.run_with_result() {
                Ok(_) => {}
                Err(err) => println!("error running a relay downloader: {}", err),
            }
        }
    }

    fn run_with_result(&self) -> Result<()> {
        let transaction = self.transaction_provider.start_transaction()?;
        let adapters = transaction.adapters();
        let registrations = adapters.registrations.borrow();

        let _pub_keys = registrations.get_pub_keys(&self.relay);
        Ok(())
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::service::app::common;
//
//    #[test]
//    fn it_works() -> Result<()> {
//        let transaction_provider = TransactionProviderMock::new();
//        let mut downloader = Downloader::new(transaction_provider);
//        match downloader.run() {
//            Ok(_) => return Err("should have failed".into()),
//            Err(_) => return Ok(()),
//        };
//
//        //match APNSToken::new(String::from("")) {
//        //    Ok(_) => panic!("constructor should have returned an error"),
//        //    Err(error) => assert_eq!(error.to_string(), String::from("empty token")),
//        //}
//    }
//
//    //fn new_sqlite() -> Result<SqliteConnectionAdapter> {
//    //    //let conn = SqliteConnectionAdapter::new(sqlite::open(":memory:")?);
//    //    //RegistrationRepositoryMigration0001::new(conn.clone()).run()?;
//    //    Ok(conn)
//    //}
//
//    struct TransactionProviderMock {}
//
//    impl TransactionProviderMock {
//        pub fn new() -> Self {
//            TransactionProviderMock {}
//        }
//    }
//
//    impl common::TransactionProvider for TransactionProviderMock {
//        fn start_transaction(&self) -> Result<Box<dyn common::Transaction>> {
//            Err("not implemented".into())
//        }
//    }
//}
