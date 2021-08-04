use serde::{Deserialize, Serialize};

use self::schema::FullSchema;

pub mod schema;
pub mod view;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbExport {
    pub schemas: Vec<FullSchema>,
}
