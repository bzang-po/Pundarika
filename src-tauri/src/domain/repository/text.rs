use crate::domain::{
    entity::text::{NewText, Text},
    value_object::search::SearchKeyword,
};
#[async_trait::async_trait]
pub trait TextRepository {
    /// 新規典籍を作成する
    ///
    /// # Arguments
    ///
    /// * `new_text` - 作成する新規典籍の情報
    async fn create(&self, new_text: &NewText) -> Result<(), Box<dyn std::error::Error>>;

    /// 典籍を削除する
    ///
    /// # Arguments
    ///
    /// * `text` - 削除する典籍の情報
    async fn delete(&self, text: &Text) -> Result<(), Box<dyn std::error::Error>>;

    /// 典籍を更新する
    ///
    /// # Arguments
    ///
    /// * `text` - 更新する典籍の情報
    async fn update(&self, texts: &Text) -> Result<(), Box<dyn std::error::Error>>;

    /// 典籍の順序を変更する
    ///
    /// # Arguments
    ///
    /// * `texts` - 新しい順序の典籍リスト
    async fn alter_order(&self, texts: &Vec<Text>) -> Result<(), Box<dyn std::error::Error>>;

    /// すべての典籍を順序通りに取得する
    async fn fetch_all_with_order(&self) -> Result<Vec<Text>, Box<dyn std::error::Error>>;

    /// キーワードに基づいて典籍を検索する
    /// # Arguments
    ///
    /// * `keywords` - 検索に使用するキーワード
    async fn search(
        &self,
        keywords: &SearchKeyword,
    ) -> Result<Vec<Text>, Box<dyn std::error::Error>>;
}
