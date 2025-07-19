use crate::db::dao::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Option<String>,
    pub name: String,
    pub role_ids: Vec<String>,
}

impl Entity for Group {
    fn table_name() -> &'static str {
        "groups"
    }
    fn id(&self) -> Option<String> {
        self.id.clone()
    }
    fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }
}
