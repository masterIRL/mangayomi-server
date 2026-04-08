use tokio::sync::OnceCell;
use mongodb::Client;

/// Global variable for the database connection
pub static CONN: OnceCell<Client> = OnceCell::const_new();

pub mod bulk_upsert;
pub mod utils;