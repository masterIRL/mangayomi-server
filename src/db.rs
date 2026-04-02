use tokio::sync::OnceCell;
use lazy_static::lazy_static;
use mongodb::Client;

lazy_static! {
    /// Global variable for the database connection
    pub static ref CONN: OnceCell<Client> = OnceCell::const_new();
}

pub mod bulk_upsert;