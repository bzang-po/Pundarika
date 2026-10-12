use crate::domain::entity::text::{NewText, Text};
use crate::domain::repository::text::TextRepository;
use crate::domain::value_object::search::SearchKeyword;
use crate::infrastructure::persistence::sqlite::service::{self, SqliteFuzzySearchTarget};
use chrono::NaiveDateTime;
use sqlx::QueryBuilder;

///
/// `Texts` テーブルに対する全文検索のターゲット構造体
///
#[derive(Debug)]
pub struct TextsFuzzySearchTarget;

impl SqliteFuzzySearchTarget for TextsFuzzySearchTarget {
    const TABLE: &'static str = "texts";
    const SELECT_COLUMNS: &'static str = "*";
    const ID_COLUMN: &'static str = "id";
    const LIKE_COLUMN: &'static str = "name";
    const FTS_TABLE: &'static str = "texts_fts";
}

///
/// `Texts` テーブルのデータを表す DTO (Data Transfer Object) 構造体
///
#[derive(Debug, Clone, sqlx::FromRow)]
struct TextDto {
    id: i64,
    name: String,
    order: i64,
    created_at: String,
}

///
/// `TextDto` から `Text` への変換を試みる実装
///
impl TryFrom<TextDto> for Text {
    type Error = Box<dyn std::error::Error>;

    fn try_from(dto: TextDto) -> Result<Self, Self::Error> {
        let created_at =
            NaiveDateTime::parse_from_str(&dto.created_at, "%Y-%m-%d %H:%M:%S")?.and_utc();
        Ok(Text {
            id: dto.id,
            name: dto.name,
            order: dto.order,
            created_at,
        })
    }
}

///
/// `SqliteTextRepository` は `TextRepository` トレイトを実装し、SQLite データベースを用いた `Text` エンティティの永続化を提供する構造体
///
pub struct SqliteTextRepository {
    pool: sqlx::SqlitePool,
}

impl SqliteTextRepository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TextRepository for SqliteTextRepository {
    ///
    /// `NewText` オブジェクトを受け取り、`texts_view` テーブルに新しいレコードを挿入する非同期関数
    ///
    async fn create(&self, new_text: &NewText) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            insert into texts_view (name)
            values ($1)
            "#,
            new_text.name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    ///
    /// `Text` オブジェクトを受け取り、対応するレコードを `texts` テーブルから削除する非同期関数
    ///
    async fn delete(&self, text: &Text) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            delete from texts
            where
                id = $1
            "#,
            text.id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, text: &Text) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            update texts
            set
                name = $1
            where
                id = $2
            "#,
            text.name,
            text.id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn alter_order(&self, texts: &Vec<Text>) -> Result<(), Box<dyn std::error::Error>> {
        let mut tx = self.pool.begin().await?;

        for text in texts {
            sqlx::query!(
                r#"
                update texts
                set
                    "order" = $1
                where
                    id = $2
                "#,
                text.order,
                text.id
            )
            .execute(tx.as_mut())
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn fetch_all_with_order(&self) -> Result<Vec<Text>, Box<dyn std::error::Error>> {
        let text_rows = sqlx::query_as!(
            TextDto,
            r#"
            select
                *
            from
                texts
            order by
                "order" asc
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let texts = text_rows
            .into_iter()
            .map(|dto| dto.try_into())
            .collect::<Result<Vec<Text>, _>>()?;
        Ok(texts)
    }

    async fn search(
        &self,
        keywords: &SearchKeyword,
    ) -> Result<Vec<Text>, Box<dyn std::error::Error>> {
        let mut builder: QueryBuilder<sqlx::Sqlite> =
            service::build_search_query::<TextsFuzzySearchTarget>(&keywords);
        let search_result_rows = builder
            .push(" order by \"order\" asc")
            .build_query_as::<TextDto>()
            .fetch_all(&self.pool)
            .await?;

        let search_result_texts = search_result_rows
            .into_iter()
            .map(|dto| dto.try_into())
            .collect::<Result<Vec<Text>, _>>()?;

        Ok(search_result_texts)
    }
}

#[cfg(test)]
mod tests {
    use super::SqliteTextRepository;
    use crate::domain::entity::text::NewText;
    use crate::domain::repository::text::TextRepository;
    use crate::domain::value_object::search::{SearchKeyword, SearchType};
    use sqlx::sqlite::SqlitePoolOptions;

    // migrationを適用した単一接続のインメモリSQLite Repositoryを作成する。
    async fn create_test_repository() -> SqliteTextRepository {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        SqliteTextRepository::new(pool)
    }

    // 指定した名称をRepository経由で登録し、viewとFTSトリガーを実行する。
    async fn insert_texts(repository: &SqliteTextRepository, names: &[&str]) {
        for name in names {
            repository
                .create(&NewText {
                    name: (*name).to_owned(),
                })
                .await
                .unwrap();
        }
    }

    // Textの一覧から比較対象となる名称だけを取り出す。
    fn names_of(texts: &[crate::domain::entity::text::Text]) -> Vec<&str> {
        texts.iter().map(|text| text.name.as_str()).collect()
    }

    // AND検索では、すべての検索語を含む典籍だけが返ることを確認する。
    #[tokio::test]
    async fn searches_matching_texts_with_and() {
        let repository = create_test_repository().await;
        insert_texts(
            &repository,
            &["日本語データベース", "日本語の本", "英語データベース"],
        )
        .await;

        let keywords = SearchKeyword::new("日本語 データベース", SearchType::And).unwrap();
        let results = repository.search(&keywords).await.unwrap();

        assert_eq!(names_of(&results), vec!["日本語データベース"]);
    }

    // OR検索では、いずれかの検索語を含む典籍が返ることを確認する。
    #[tokio::test]
    async fn searches_matching_texts_with_or() {
        let repository = create_test_repository().await;
        insert_texts(
            &repository,
            &["日本語データベース", "日本語の本", "英語データベース"],
        )
        .await;

        let keywords = SearchKeyword::new("日本語 データベース", SearchType::Or).unwrap();
        let results = repository.search(&keywords).await.unwrap();

        assert_eq!(
            names_of(&results),
            vec!["日本語データベース", "日本語の本", "英語データベース"]
        );
    }

    // 2文字以下のLIKE検索語と3文字以上のFTS検索語を組み合わせられることを確認する。
    #[tokio::test]
    async fn searches_mixed_short_and_long_keywords() {
        let repository = create_test_repository().await;
        insert_texts(
            &repository,
            &[
                "ab日本語データベース",
                "ab日本語の本",
                "cd日本語データベース",
            ],
        )
        .await;

        let keywords = SearchKeyword::new("ab 日本語", SearchType::And).unwrap();
        let results = repository.search(&keywords).await.unwrap();

        assert_eq!(
            names_of(&results),
            vec!["ab日本語データベース", "ab日本語の本"]
        );
    }

    // 登録されていない検索語では結果が返らないことを確認する。
    #[tokio::test]
    async fn does_not_return_texts_for_uninserted_keywords() {
        let repository = create_test_repository().await;
        insert_texts(&repository, &["日本語データベース", "日本語の本"]).await;

        let keywords = SearchKeyword::new("存在しない", SearchType::Or).unwrap();
        let results = repository.search(&keywords).await.unwrap();

        assert!(results.is_empty());
    }

    // 登録した典籍をorderの昇順で取得できることを確認する。
    #[tokio::test]
    async fn fetches_all_texts_in_order() {
        let repository = create_test_repository().await;
        insert_texts(&repository, &["first", "second", "third"]).await;

        let results = repository.fetch_all_with_order().await.unwrap();

        assert_eq!(names_of(&results), vec!["first", "second", "third"]);
    }

    // 典籍名を更新すると、新しい名前で検索でき古い名前では検索できないことを確認する。
    #[tokio::test]
    async fn updates_text_name_and_fts_index() {
        let repository = create_test_repository().await;
        insert_texts(&repository, &["old name"]).await;

        let mut text = repository
            .fetch_all_with_order()
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        text.name = "new name".to_owned();
        repository.update(&text).await.unwrap();

        let old_keyword = SearchKeyword::new("old", SearchType::Or).unwrap();
        let new_keyword = SearchKeyword::new("new", SearchType::Or).unwrap();
        assert!(repository.search(&old_keyword).await.unwrap().is_empty());
        assert_eq!(
            names_of(&repository.search(&new_keyword).await.unwrap()),
            vec!["new name"]
        );
    }

    // 典籍を削除すると、一覧とFTS検索の両方から除外されることを確認する。
    #[tokio::test]
    async fn deletes_text_and_fts_index() {
        let repository = create_test_repository().await;
        insert_texts(&repository, &["to delete", "to keep"]).await;

        let text_to_delete = repository
            .fetch_all_with_order()
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        repository.delete(&text_to_delete).await.unwrap();

        let keyword = SearchKeyword::new("delete", SearchType::Or).unwrap();
        assert!(repository.search(&keyword).await.unwrap().is_empty());
        assert_eq!(
            names_of(&repository.fetch_all_with_order().await.unwrap()),
            vec!["to keep"]
        );
    }

    // 複数典籍のorderを変更すると、指定した順序で一覧取得できることを確認する。
    #[tokio::test]
    async fn alters_text_order() {
        let repository = create_test_repository().await;
        insert_texts(&repository, &["first", "second", "third"]).await;

        let mut texts = repository.fetch_all_with_order().await.unwrap();
        texts[0].order = 3;
        texts[1].order = 1;
        texts[2].order = 2;
        repository.alter_order(&texts).await.unwrap();

        assert_eq!(
            names_of(&repository.fetch_all_with_order().await.unwrap()),
            vec!["second", "third", "first"]
        );
    }
}
