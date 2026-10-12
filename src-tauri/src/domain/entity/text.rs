use chrono::{DateTime, Utc};

///
///  典籍Entity
///
#[derive(Debug)]
pub struct Text {
    pub id: i64,
    pub name: String,
    pub order: i64,
    pub created_at: DateTime<Utc>,
}

///
///  新規典籍Entity
///  IDや作成日時は存在しない
#[derive(Debug)]
pub struct NewText {
    pub name: String,
}
