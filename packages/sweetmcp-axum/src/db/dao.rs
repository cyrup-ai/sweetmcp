use std::{fmt::Debug, marker::PhantomData};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

use super::client::DatabaseClient;

/// Generic entity trait for database objects
pub trait Entity: Serialize + DeserializeOwned + Debug + Send + Sync + Clone {
    /// Get the table name for this entity
    fn table_name() -> &'static str;

    /// Get the ID of this entity
    fn id(&self) -> Option<String>;

    /// Set the ID of this entity
    fn set_id(&mut self, id: String);

    /// Generate a unique ID for this entity
    fn generate_id() -> String {
        format!("{}:{}", Self::table_name(), Uuid::new_v4())
    }
}

/// Base DAO trait providing common CRUD operations for entities
pub trait BaseDao {
    type Entity: crate::db::dao::Entity + 'static;

    /// Create a new entity
    fn create(&self, entity: &mut Self::Entity) -> crate::types::AsyncTask<Self::Entity>;

    /// Find a single entity by ID
    fn find_by_id(&self, id: &str) -> crate::types::AsyncTask<Option<Self::Entity>>;

    /// Update an entity
    fn update(&self, entity: &Self::Entity) -> crate::types::AsyncTask<Option<Self::Entity>>;

    /// Delete an entity by ID
    fn delete(&self, id: &str) -> crate::types::AsyncTask<Option<Self::Entity>>;

    /// Find all entities as a stream
    fn find(&self) -> crate::types::AsyncTask<crate::types::AsyncStream<Self::Entity>>;

    /// Create a table for this entity
    fn create_table(&self) -> crate::types::AsyncTask<()>;
}

/// Base Data Access Object for SurrealDB
#[derive(Debug, Clone)]
pub struct Dao<T: Entity> {
    client: DatabaseClient,
    _marker: PhantomData<T>,
}

impl<T: Entity + 'static> Dao<T> {
    /// Create a new DAO
    pub fn new(client: DatabaseClient) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }

    /// Get the client reference
    pub fn client(&self) -> &DatabaseClient {
        &self.client
    }

    /// Find a single entity by ID
    pub fn find_by_id(&self, id: &str) -> crate::types::AsyncTask<Option<T>> {
        let client = self.client.clone();
        let id = id.to_string();
        crate::types::AsyncTask::from_future(async move {
            let id_parts: Vec<&str> = id.split(':').collect();
            if id_parts.len() != 2 {
                return None;
            }
            let table = id_parts[0];
            if table != T::table_name() {
                return None;
            }
            let task = client.find_by_id::<T>(T::table_name(), &id);
            task.await.unwrap_or(None)
        })
    }

    /// Find all entities
    pub fn find(&self) -> crate::types::AsyncTask<crate::types::AsyncStream<T>> {
        let client = self.client.clone();
        crate::types::AsyncTask::from_future(async move { client.find::<T>(T::table_name()).await })
    }

    /// Create a new entity
    pub fn create(
        &self,
        entity: &mut <Self as BaseDao>::Entity,
    ) -> crate::types::AsyncTask<<Self as BaseDao>::Entity> {
        let client = self.client.clone();
        let mut entity_clone = entity.clone();
        if entity_clone.id().is_none() {
            let generated_id = T::generate_id();
            entity_clone.set_id(generated_id.clone());
            entity.set_id(generated_id);
        }
        crate::types::AsyncTask::from_future(async move {
            let task = client.create::<T>(T::table_name(), entity_clone.clone());
            task.await.expect("Failed to create entity")
        })
    }

    /// Update an entity
    pub fn update(
        &self,
        entity: &<Self as BaseDao>::Entity,
    ) -> crate::types::AsyncTask<Option<<Self as BaseDao>::Entity>> {
        let client = self.client.clone();
        let entity = entity.clone();
        crate::types::AsyncTask::from_future(async move {
            match entity.id() {
                Some(id) => {
                    let id_parts: Vec<&str> = id.split(':').collect();
                    if id_parts.len() != 2 {
                        return None;
                    }
                    let task = client.update::<T>(T::table_name(), &id, entity.clone());
                    task.await.unwrap_or(None)
                }
                None => None,
            }
        })
    }

    /// Delete an entity by ID
    pub fn delete(&self, id: &str) -> crate::types::AsyncTask<Option<<Self as BaseDao>::Entity>> {
        let client = self.client.clone();
        let id = id.to_string();
        crate::types::AsyncTask::from_future(async move {
            let id_parts: Vec<&str> = id.split(':').collect();
            if id_parts.len() != 2 {
                return None;
            }
            let table = id_parts[0];
            if table != T::table_name() {
                return None;
            }
            let task = client.delete::<T>(T::table_name(), &id);
            task.await.unwrap_or(None)
        })
    }

    /// Create a table for this entity
    pub fn create_table(&self) -> crate::types::AsyncTask<()> {
        let client = self.client.clone();
        crate::types::AsyncTask::from_future(async move {
            let query = format!("DEFINE TABLE {} SCHEMAFULL", T::table_name());
            let task = client.query::<()>(&query);
            let _ = task.await;
        })
    }
}

impl<T: Entity + 'static> BaseDao for Dao<T> {
    type Entity = T;
    /// Create a new entity
    fn create(
        &self,
        entity: &mut <Self as BaseDao>::Entity,
    ) -> crate::types::AsyncTask<<Self as BaseDao>::Entity> {
        let client = self.client.clone();
        let mut entity_clone = entity.clone();
        if entity_clone.id().is_none() {
            let generated_id = T::generate_id();
            entity_clone.set_id(generated_id.clone());
            entity.set_id(generated_id);
        }
        crate::types::AsyncTask::from_future(async move {
            let task = client.create::<T>(T::table_name(), entity_clone.clone());
            task.await.expect("Failed to create entity")
        })
    }

    /// Find a single entity by ID
    fn find_by_id(&self, id: &str) -> crate::types::AsyncTask<Option<<Self as BaseDao>::Entity>> {
        let client = self.client.clone();
        let id = id.to_string();
        crate::types::AsyncTask::from_future(async move {
            let id_parts: Vec<&str> = id.split(':').collect();
            if id_parts.len() != 2 {
                return None;
            }
            let table = id_parts[0];
            if table != T::table_name() {
                return None;
            }
            let task = client.find_by_id::<T>(T::table_name(), &id);
            task.await.unwrap_or(None)
        })
    }

    /// Update an entity
    fn update(
        &self,
        entity: &<Self as BaseDao>::Entity,
    ) -> crate::types::AsyncTask<Option<<Self as BaseDao>::Entity>> {
        let client = self.client.clone();
        let entity = entity.clone();
        crate::types::AsyncTask::from_future(async move {
            match entity.id() {
                Some(id) => {
                    let id_parts: Vec<&str> = id.split(':').collect();
                    if id_parts.len() != 2 {
                        return None;
                    }
                    let task = client.update::<T>(T::table_name(), &id, entity.clone());
                    task.await.unwrap_or(None)
                }
                None => None,
            }
        })
    }

    /// Delete an entity by ID
    fn delete(&self, id: &str) -> crate::types::AsyncTask<Option<<Self as BaseDao>::Entity>> {
        let client = self.client.clone();
        let id = id.to_string();
        crate::types::AsyncTask::from_future(async move {
            let id_parts: Vec<&str> = id.split(':').collect();
            if id_parts.len() != 2 {
                return None;
            }
            let table = id_parts[0];
            if table != T::table_name() {
                return None;
            }
            let task = client.delete::<T>(T::table_name(), &id);
            task.await.unwrap_or(None)
        })
    }

    /// Find all entities as a stream
    fn find(&self) -> crate::types::AsyncTask<crate::types::AsyncStream<T>> {
        let client = self.client.clone();
        crate::types::AsyncTask::from_future(async move { client.find::<T>(T::table_name()).await })
    }

    /// Create a table for this entity
    fn create_table(&self) -> crate::types::AsyncTask<()> {
        let client = self.client.clone();
        crate::types::AsyncTask::from_future(async move {
            let query = format!("DEFINE TABLE {} SCHEMAFULL", T::table_name());
            let task = client.query::<()>(&query);
            let _ = task.await;
        })
    }
}

/// Common fields for database entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    /// Entity ID
    pub id: Option<String>,

    /// Creation timestamp
    #[serde(default = "utc_now")]
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
}

impl BaseEntity {
    /// Create a new entity
    pub fn new() -> Self {
        Self {
            id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to get the current UTC time
fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

/// Example user entity implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(flatten)]
    base: BaseEntity,

    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
}

impl Entity for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn id(&self) -> Option<String> {
        self.base.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.base.id = Some(id);
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            base: BaseEntity::new(),
            username: String::new(),
            email: String::new(),
            password_hash: None,
        }
    }
}
