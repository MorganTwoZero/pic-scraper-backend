mod extract;
mod save;
pub mod startup;
mod transform;

use errors::Error;
pub use extract::fill_db;
pub use transform::{Post, PostSource};

type Result<T, E = Error> = std::result::Result<T, E>;
