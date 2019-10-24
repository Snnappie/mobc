use futures_01::{Future as _, Stream as _};
use mobc::futures::{compat::Future01CompatExt, TryFutureExt};
use mobc::AnyFuture;
use mobc::ConnectionManager;
use mobc::Stream01CompatExt;
pub use tokio_postgres;
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::Client;
use tokio_postgres::Config;
use tokio_postgres::Error;
use tokio_postgres::Socket;

pub struct PostgresConnectionManager<Tls> {
    config: Config,
    tls: Tls,
    executor: DefaultExecutor,
}

impl<Tls> PostgresConnectionManager<Tls> {
    pub fn new(config: Config, tls: Tls) -> Self {
        PostgresConnectionManager {
            config: config,
            tls: tls,
        }
    }
}

impl<Tls> ConnectionManager for PostgresConnectionManager<Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    type Connection = Client;
    type 
    type Error = Error;

    fn connect(&self) -> AnyFuture<Self::Connection, Self::Error> {
        Box::new(
            self.config
                .connect(self.tls.clone())
                .compat()
                .map_ok(|(client, conn)| client),
        )
    }

    fn is_valid(&self, mut conn: Self::Connection) -> AnyFuture<Self::Connection, Self::Error> {
        Box::new(
            conn.simple_query("")
                .collect()
                .then(move |r| match r {
                    Ok(_) => Ok(conn),
                    Err(e) => Err(e),
                })
                .compat(),
        )
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.is_closed()
    }
}