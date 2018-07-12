#[macro_use(try_future)]
extern crate boxfuture;
#[cfg(feature = "orm")]
#[macro_use]
extern crate diesel;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(feature = "sql")]
extern crate postgres;
#[cfg(feature = "orm")]
extern crate r2d2_diesel;
#[cfg(feature = "sql")]
extern crate r2d2_postgres;
extern crate regex;
extern crate tokio;

pub mod github;

#[cfg(feature = "orm")]
mod orm;
#[cfg(feature = "orm")]
pub use orm::{comments_for_line, connect, leave_comment, DatabaseConnection};
#[cfg(feature = "sql")]
mod sql;
#[cfg(feature = "sql")]
pub use sql::{comments_for_line, connect, leave_comment, DatabaseConnection};
mod models;
pub use models::*;

#[cfg(feature = "sql")]
pub type ConnectionManager = r2d2_postgres::PostgresConnectionManager;
#[cfg(feature = "orm")]
pub type ConnectionManager = r2d2_diesel::ConnectionManager<diesel::PgConnection>;

#[cfg(feature = "orm")]
pub fn make_connection_manager(database: &str) -> ConnectionManager {
    ConnectionManager::new(database)
}

#[cfg(feature = "sql")]
pub fn make_connection_manager(database: &str) -> ConnectionManager {
    ConnectionManager::new(database, r2d2_postgres::TlsMode::None)
        .expect("Failed to create database manager")
}
