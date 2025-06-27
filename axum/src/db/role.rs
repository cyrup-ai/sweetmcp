use serde::{Serialize, Deserialize};
use crate::db::dao::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Option<String>,
    pub name: String,
    pub permissions: Vec<String>,
}

impl Entity for Role {
    fn table_name() -> &'static str { "roles" }
    fn id(&self) -> Option<String> { self.id.clone() }
    fn set_id(&mut self, id: String) { self.id = Some(id); }
}
