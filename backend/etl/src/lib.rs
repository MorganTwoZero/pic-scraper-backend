pub mod extract;
mod load;
mod transform;

use errors::Error;
pub use extract::{create_vec_posts, fill_db};
pub use load::{load_honkai_posts, load_twitter_home_posts};
pub use transform::Post;

pub type Result<T, E = Error> = std::result::Result<T, E>;
