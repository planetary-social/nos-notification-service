use crate::errors::Result;
use crate::service::app::common;
use crate::service::domain;
use nostr::message::client;
use nostr::message::subscription;
use std::collections::HashMap;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use tungstenite::Message as WsMessage;

pub struct Downloader<'a, T> {
    transaction_provider: T,
    relay_downloaders: HashMap<domain::RelayAddress, RelayDownloader<'a>>,
}

impl<'a, T> Downloader<'a, T> {
    pub fn new(transaction_provider: T) -> Self {
        Self {
            transaction_provider,
            relay_downloaders: HashMap::new(),
        }
    }
}

impl<'a, T> Downloader<'a, T>
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

pub struct RelayDownloader<'scope> {
    sender: SyncSender<()>,
    handle: Option<thread::ScopedJoinHandle<'scope, ()>>,
}

impl<'scope> RelayDownloader<'scope> {
    fn new<T>(
        scope: &'scope thread::Scope<'scope, '_>,
        relay: domain::RelayAddress,
        transaction_provider: T,
    ) -> RelayDownloader<'scope>
    where
        T: common::TransactionProvider + Clone + Sync + Send + 'scope,
    {
        let (sender, receiver) = sync_channel(0);

        let handle = scope.spawn(|| {
            RelayDownloaderRunner::new(relay, transaction_provider, receiver).run();
        });

        Self {
            sender,
            handle: Some(handle),
        }
    }
}

impl<'scope> Drop for RelayDownloader<'scope> {
    fn drop(&mut self) {
        self.sender
            .send(())
            .expect("could not notify the thread that it is time to terminate?!");
        self.handle
            .take()
            .unwrap()
            .join()
            .expect("thread could not be joined?!");
    }
}

pub struct RelayDownloaderRunner<T> {
    transaction_provider: T,
    relay: domain::RelayAddress,
    receiver: Receiver<()>,
}

impl<T> RelayDownloaderRunner<T>
where
    T: common::TransactionProvider,
{
    fn new(
        relay: domain::RelayAddress,
        transaction_provider: T,
        receiver: Receiver<()>,
    ) -> RelayDownloaderRunner<T> {
        Self {
            transaction_provider,
            relay,
            receiver,
        }
    }

    fn run(&self) {
        loop {
            match self.run_with_result() {
                Ok(_) => {}
                Err(err) => println!("error running a relay downloader: {}", err),
            }

            self.receiver.recv().unwrap();
        }
    }

    // todo move nostr low-level transport code somewhere else
    fn run_with_result(&self) -> Result<()> {
        let transaction = self.transaction_provider.start_transaction()?;
        let adapters = transaction.adapters();
        let registrations = adapters.registrations.borrow();

        let (mut socket, _) = tungstenite::connect(self.relay.as_ref())?;

        for pub_key_info in registrations.get_pub_keys(&self.relay)? {
            let pub_key_hex = pub_key_info.pub_key().hex();
            let filters = vec![subscription::Filter::new().author(pub_key_hex.clone())];

            let msg = client::ClientMessage::new_req(
                subscription::SubscriptionId::new(pub_key_hex),
                filters,
            )
            .as_json();
            socket
                .write_message(WsMessage::Text(msg))
                .expect("Impossible to send message");
        }

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
