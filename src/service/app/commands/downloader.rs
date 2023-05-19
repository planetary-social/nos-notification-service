use crate::errors::Result;
use crate::service::app::common;

pub struct Downloader<T> {
    _transaction_provider: T,
    //relay_downloaders: HashMap<domain::RelayAddress, RelayDownloader<T, R1, R2>>,
}

impl<T> Downloader<T> {
    pub fn new(transaction_provider: T) -> Self {
        Self {
            _transaction_provider: transaction_provider,
        }
    }
}

impl<T> Downloader<T>
where
    T: common::TransactionProvider,
{
    fn _run(&mut self) -> Result<()> {
        let transaction = self._transaction_provider.start_transaction()?;

        let adapters = transaction.adapters();
        let registrations = adapters.registrations.borrow();

        let initial_relays = registrations.get_relays()?;

        //let initial_relays: Vec<domain::RelayAddress> = self
        //    .transaction_provider
        //    .transact(|adapters| adapters.registrations.get_relays())
        //    .unwrap(); // todo don't unwrap

        for _initial_relay in initial_relays {
            //let v = RelayDownloader::new(initial_relay.clone(), self.transaction_provider);
            //self.relay_downloaders.insert(initial_relay.clone(), v);
        }

        // todo update the list in a loop

        Err("not implemented".into())
    }
}

//pub struct RelayDownloader<T, R1, R2> {
//    transaction_provider: T,
//    r1: PhantomData<R1>,
//    r2: PhantomData<R2>,
//
//    relay: domain::RelayAddress,
//
//    //run_join: thread::JoinHandle<()>,
//    tx: mpsc::Sender<()>,
//}
//
//impl<T, R1, R2> RelayDownloader<T, R1, R2> {
//    fn new<R>(relay: domain::RelayAddress, transaction_provider: T) -> RelayDownloader<T, R1, R2>
//    where
//        R1: common::RegistrationRepository,
//        R2: common::EventRepository,
//        T: common::TransactionProvider,
//    {
//        let (tx, rx) = mpsc::channel::<()>();
//
//        //let h = thread::spawn(|| {
//        //    Self::run(relay, transaction_provider, rx);
//        //});
//
//        let v = RelayDownloader {
//            transaction_provider,
//            r1: PhantomData {},
//            r2: PhantomData {},
//            relay,
//            //run_join: h,
//            tx,
//        };
//
//        return v;
//    }
//
//    fn run(relay: domain::RelayAddress, transaction_provider: T, rx: mpsc::Receiver<()>) {
//        let initial_pub_keys: Vec<domain::RelayAddress> = Vec::new();
//
//        //let c = |adapters: Adapters<R1, R2>| -> Result<()> {
//        //    let relays = adapters.registrations.get_relays()?;
//
//        //    for relay in relays {
//        //        initial_relays.push(relay);
//        //    }
//
//        //    return Ok(());
//        //};
//
//        //for initial_relay in initial_relays {
//        //}
//
//        //self.transaction_provider.transact(&c);
//    }
//}

//impl<T, R1, R2> Drop for RelayDownloader<T, R1, R2>
//where
//    T: common::TransactionProvider<common::Adapters<R1, R2>, R>,
//    R1: common::RegistrationRepository,
//    R2: common::EventRepository,
//{
//    fn drop(&mut self) {
//        self.tx.send(());
//        self.run_join.join();
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::app::common;

    #[test]
    fn it_works() -> Result<()> {
        let transaction_provider = TransactionProviderMock::new();
        let mut downloader = Downloader::new(transaction_provider);
        match downloader._run() {
            Ok(_) => return Err("should have failed".into()),
            Err(_) => return Ok(()),
        };

        //match APNSToken::new(String::from("")) {
        //    Ok(_) => panic!("constructor should have returned an error"),
        //    Err(error) => assert_eq!(error.to_string(), String::from("empty token")),
        //}
    }

    //fn new_sqlite() -> Result<SqliteConnectionAdapter> {
    //    //let conn = SqliteConnectionAdapter::new(sqlite::open(":memory:")?);
    //    //RegistrationRepositoryMigration0001::new(conn.clone()).run()?;
    //    Ok(conn)
    //}

    struct TransactionProviderMock {}

    impl TransactionProviderMock {
        pub fn new() -> Self {
            TransactionProviderMock {}
        }
    }

    impl common::TransactionProvider for TransactionProviderMock {
        fn start_transaction(&self) -> Result<Box<dyn common::Transaction>> {
            Err("not implemented".into())
        }
    }
}
