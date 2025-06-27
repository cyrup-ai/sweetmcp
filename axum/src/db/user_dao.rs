use crate::db::{Dao, BaseDao, User, DatabaseClient};

#[derive(Clone)]
pub struct UserDao {
    inner: Dao<User>,
}

impl UserDao {
    pub fn new(client: DatabaseClient) -> Self {
        Self { inner: Dao::new(client) }
    }

    pub fn find_by_id(&self, id: &str) -> crate::types::AsyncTask<Option<User>> {
        self.inner.find_by_id(id)
    }

    pub fn find(&self) -> crate::types::AsyncTask<EntityStream<User>> {
        self.inner.find()
    }

    // You can add more User-specific methods here
}
