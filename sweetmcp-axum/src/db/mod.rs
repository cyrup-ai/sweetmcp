pub mod client;
pub mod config;
pub mod dao;
pub mod error;
pub mod group;
pub mod result;
pub mod role;
pub mod user;

// Re-export main components
pub use client::{DatabaseClient, connect_database};
pub use config::{DatabaseConfig, StorageEngine};
pub use dao::{BaseDao, Dao, Entity};
pub use error::{SurrealdbError, SurrealdbErrorContext};
pub use surrealdb::Surreal;
// Export common SurrealDB types for convenience
pub use group::Group;
pub use role::Role;
pub use surrealdb::sql::{Array, Id, Object, Thing, Value};
pub use user::User;
