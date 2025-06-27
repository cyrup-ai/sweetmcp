/// Type alias for results with SurrealdbError
use crate::db::error::SurrealdbError;
pub type Result<T> = std::result::Result<T, SurrealdbError>;
