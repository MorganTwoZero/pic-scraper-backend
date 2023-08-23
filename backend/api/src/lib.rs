mod load;
mod site_routes;
pub mod startup;

use embed;
use errors::Error;

type Result<T, E = Error> = std::result::Result<T, E>;
