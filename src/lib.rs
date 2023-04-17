pub mod config;
pub mod embed;
pub mod errors;
pub mod etl;
pub mod site_routes;
pub mod startup;
pub mod telemetry;
pub mod utils;

pub use errors::{Error, ResultExt};

pub type Result<T, E = Error> = std::result::Result<T, E>;
