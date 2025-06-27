use serde::{Serialize, Deserialize};
use crate::db::dao::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub group_id: Option<String>,
}

impl Entity for User {
    fn table_name() -> &'static str { "users" }
    fn id(&self) -> Option<String> { self.id.clone() }
    fn set_id(&mut self, id: String) { self.id = Some(id); }
}
