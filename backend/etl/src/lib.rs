mod extract;
mod save;
pub mod startup;
mod transform;

use errors::Error;
pub use transform::{Post, PostSource};
pub use extract::fill_db;

type Result<T, E = Error> = std::result::Result<T, E>;
